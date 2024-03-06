use chrono::{DateTime, Utc};
use std::time::SystemTime;
use tonic::{Request, Response, Status};
use tracing::instrument;

use qcdn_proto_server::{
    qcdn_general_server::QcdnGeneral, PingMessage, VersionResponse, PACKAGE_VERSION,
};

use crate::error::Result;

const DATETIME_FORMAT: &str = "%d/%m/%Y %T:%f";

#[derive(Debug, Default)]
pub struct GeneralService {}

#[tonic::async_trait]
impl QcdnGeneral for GeneralService {
    #[instrument(skip(self))]
    async fn ping(&self, request: Request<PingMessage>) -> Result<Response<PingMessage>, Status> {
        let now = SystemTime::now();
        let now_datetime: DateTime<Utc> = now.into();

        let from = request
            .into_inner()
            .timestamp
            .and_then(|t| DateTime::from_timestamp(t.seconds, u32::try_from(t.nanos).unwrap_or(0)))
            .map(|d| d.format(DATETIME_FORMAT).to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        tracing::info!(
            "Ping request from {} at {}",
            from,
            now_datetime.format(DATETIME_FORMAT)
        );

        let reply = PingMessage {
            timestamp: Some(now.into()),
        };
        Ok(Response::new(reply))
    }

    #[instrument(skip(self))]
    async fn version(&self, _: Request<()>) -> Result<Response<VersionResponse>, Status> {
        tracing::info!("Got Version request");
        let reply = VersionResponse {
            version: PACKAGE_VERSION.to_string(),
        };

        Ok(Response::new(reply))
    }
}
