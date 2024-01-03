use std::time::SystemTime;

use tonic::{Request, Response, Status};

use crate::constants::VERSION;

use super::{qcdn_general_server::QcdnGeneral, PingMessage, VersionResponse};

#[derive(Debug, Default)]
pub struct GeneralService {}

#[tonic::async_trait]
impl QcdnGeneral for GeneralService {
    async fn ping(&self, request: Request<PingMessage>) -> Result<Response<PingMessage>, Status> {
        let ts = SystemTime::now();
        tracing::info!(
            "Ping request from {:?} at {:?}",
            request.into_inner().timestamp,
            ts
        );

        let reply = PingMessage {
            timestamp: Some(ts.into()),
        };
        Ok(Response::new(reply))
    }
    async fn version(&self, _: Request<()>) -> Result<Response<VersionResponse>, Status> {
        let reply = VersionResponse {
            version: VERSION.to_string(),
        };
        tracing::info!("Got Version request");

        Ok(Response::new(reply))
    }
}
