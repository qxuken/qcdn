use std::time::SystemTime;

use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use color_eyre::Result;
use qcdn_proto_client::{
    qcdn_file_queries_client::QcdnFileQueriesClient, qcdn_general_client::QcdnGeneralClient,
    GetDirResponse, GetFileResponse, GetFileVersionResponse, GetFileVersionsRequest,
    GetFilesRequest, PingMessage, PACKAGE_VERSION,
};
use tonic::{transport::Channel, Request};
use tracing::instrument;

use crate::cli::Cli;

#[derive(Debug)]
pub struct Rpc {
    url: String,
}

impl From<&Cli> for Rpc {
    fn from(value: &Cli) -> Self {
        Self {
            url: value.url.clone(),
        }
    }
}

impl Rpc {
    #[instrument]
    pub async fn connect_to_general(&self) -> Result<QcdnGeneralClient<Channel>> {
        tracing::trace!("Establishing general connection");
        let general = QcdnGeneralClient::connect(self.url.clone()).await?;

        Ok(general)
    }

    #[instrument(skip_all)]
    pub async fn latency(general: &mut QcdnGeneralClient<Channel>) -> Result<Option<TimeDelta>> {
        let send_ts = SystemTime::now();
        tracing::debug!("Sending ping at {send_ts:?}");
        let ping = PingMessage {
            timestamp: Some(send_ts.into()),
        };

        let latency = general
            .ping(Request::new(ping))
            .await?
            .into_inner()
            .timestamp
            .and_then(|t| {
                NaiveDateTime::from_timestamp_opt(t.seconds, u32::try_from(t.nanos).unwrap_or(0))
            })
            .map(|dt| {
                let send_datetime: DateTime<Utc> = send_ts.into();
                dt - send_datetime.naive_utc()
            });

        Ok(latency)
    }

    #[instrument(skip_all)]
    pub async fn server_version(general: &mut QcdnGeneralClient<Channel>) -> Result<String> {
        tracing::debug!("Sending Version request");

        let res = general.version(Request::new(())).await?;

        Ok(res.into_inner().version)
    }

    #[instrument]
    pub fn client_version() -> &'static str {
        PACKAGE_VERSION
    }
}

impl Rpc {
    #[instrument]
    pub async fn connect_to_file_query(&self) -> Result<QcdnFileQueriesClient<Channel>> {
        tracing::trace!("Establishing file query connection");
        let general = QcdnFileQueriesClient::connect(self.url.clone()).await?;

        Ok(general)
    }

    #[instrument(skip_all)]
    pub async fn list_dirs(
        files: &mut QcdnFileQueriesClient<Channel>,
    ) -> Result<Vec<GetDirResponse>> {
        tracing::debug!("Sending GetDirs request");

        let res = files.get_dirs(Request::new(())).await?.into_inner();

        Ok(res.items)
    }

    #[instrument(skip(file_query))]
    pub async fn list_files(
        file_query: &mut QcdnFileQueriesClient<Channel>,
        dir_id: i64,
    ) -> Result<Vec<GetFileResponse>> {
        tracing::debug!("Sending GetFiles request");

        let req = GetFilesRequest { dir_id };
        let res = file_query.get_files(Request::new(req)).await?.into_inner();

        Ok(res.items)
    }

    #[instrument(skip(file_query))]
    pub async fn list_versions(
        file_query: &mut QcdnFileQueriesClient<Channel>,
        file_id: i64,
    ) -> Result<Vec<GetFileVersionResponse>> {
        tracing::debug!("Sending GetFiles request");

        let req = GetFileVersionsRequest { file_id };
        let res = file_query
            .get_file_versions(Request::new(req))
            .await?
            .into_inner();

        Ok(res.items)
    }
}
