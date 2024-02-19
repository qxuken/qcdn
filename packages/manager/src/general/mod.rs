use std::time::SystemTime;

use qcdn_proto_server::{
    qcdn_general_server::QcdnGeneral, PingMessage, VersionResponse, PACKAGE_VERSION,
};
use tonic::{Request, Response, Status};
use tracing::instrument;

#[derive(Debug, Default)]
pub struct GeneralService {}

#[tonic::async_trait]
impl QcdnGeneral for GeneralService {
    #[instrument(skip(self))]
    async fn ping(&self, request: Request<PingMessage>) -> Result<Response<PingMessage>, Status> {
        let ts = SystemTime::now();
        tracing::info!(
            "Ping request from {:?} at {ts:?}",
            request.into_inner().timestamp,
        );

        let reply = PingMessage {
            timestamp: Some(ts.into()),
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
