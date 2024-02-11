use std::{pin::Pin, sync::Arc};

use async_channel::Receiver;
use chrono::DateTime;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};

use crate::{
    database::files::sync::FileSync,
    grpc::{
        qcdn_nodes_server::QcdnNodes, ConnectionRequest, GetClosestUrlRequest,
        GetClosestUrlResponse, SyncMessage,
    },
    AppState,
};

#[derive(Debug, Clone)]
pub struct NodesService {
    app_state: Arc<AppState>,
    sync: Receiver<SyncMessage>,
}

impl NodesService {
    pub fn new(app_state: Arc<AppState>, sync: Receiver<SyncMessage>) -> Self {
        Self { app_state, sync }
    }
}

#[tonic::async_trait]
impl QcdnNodes for NodesService {
    type ConnectNodeStream = Pin<Box<dyn Stream<Item = Result<SyncMessage, Status>> + Send>>;

    async fn connect_node(
        &self,
        request: Request<ConnectionRequest>,
    ) -> Result<Response<Self::ConnectNodeStream>, Status> {
        let ts = request.into_inner().timestamp.and_then(|ts| {
            DateTime::from_timestamp(ts.seconds, ts.nanos.try_into().unwrap_or_default())
        });

        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let in_updates = self.sync.clone();

        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            if let Some(ts) = ts {
                let mut updates = FileSync::uploaded_from_ts(&mut connection, &ts).await?;
                updates.extend(FileSync::tagged_from_ts(&mut connection, &ts).await?);
                updates.extend(FileSync::deleted_from_ts(&mut connection, &ts).await?);
                updates.sort_by_key(|u| u.timestamp);

                for update in updates {
                    tx.send(Ok(update.into())).await?;
                }
            }
            while let Ok(update) = in_updates.recv().await {
                tx.send(Ok(update)).await?;
            }
            anyhow::Ok(())
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream)))
    }

    async fn get_closest_url(
        &self,
        _request: Request<GetClosestUrlRequest>,
    ) -> Result<Response<GetClosestUrlResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}
