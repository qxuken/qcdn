use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use crate::{
    grpc::{
        qcdn_files_server::QcdnFiles, upload_file_request, DeleteFileVersionRequest,
        GetClosestUrlRequest, GetClosestUrlResponse, UploadFileMeta, UploadFileRequest,
        UploadFileResponse,
    },
    AppState,
};

#[derive(Debug)]
pub struct FilesService {
    app_state: AppState,
}

impl FilesService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

impl FilesService {
    async fn fallback_file_creation(&self, meta: &UploadFileMeta) -> Result<(), Status> {
        self.app_state
            .storage
            .remove_file(&meta.dir, &meta.name)
            .await
            .map_err(|e| Status::internal(e.to_string()))
    }
}

#[tonic::async_trait]
impl QcdnFiles for FilesService {
    async fn upload_file(
        &self,
        request: Request<Streaming<UploadFileRequest>>,
    ) -> Result<Response<UploadFileResponse>, Status> {
        tracing::info!("Upload file stream initiated");
        let mut in_stream = request.into_inner();

        let mut bytes = 0u64;
        let mut data = None;
        // let mut connection = self
        //     .app_state
        //     .db
        //     .connect()
        //     .await
        //     .map_err(|e| Status::internal(e.to_string()))?;

        while let Some(result) = in_stream.next().await {
            match result.map(|r| r.request).ok().flatten() {
                Some(req) => match req {
                    upload_file_request::Request::Meta(meta) => {
                        let file = self
                            .app_state
                            .storage
                            .create_file(&meta.dir, &meta.name)
                            .await
                            .map_err(|e| Status::internal(e.to_string()))?;
                        if let Err(e) = file.set_len(meta.size).await {
                            self.fallback_file_creation(&meta).await?;
                            return Err(Status::internal(e.to_string()));
                        }

                        tracing::info!("Starting upload {meta:?}");
                        data = Some((meta, file));
                    }
                    upload_file_request::Request::Part(part) => {
                        let (meta, file) = data.as_mut().unwrap();
                        bytes += part.bytes.len() as u64;
                        if let Err(e) = file.write(&part.bytes).await {
                            self.fallback_file_creation(meta).await?;
                            return Err(Status::internal(e.to_string()));
                        }
                    }
                },
                None => {
                    if let Some((meta, _file)) = data.as_ref() {
                        self.fallback_file_creation(meta).await?;
                    }
                    tracing::error!("error")
                }
            };
        }

        tracing::info!("Upload file stream ended");

        if let Some((meta, _file)) = data.filter(|(m, _)| m.size != bytes) {
            self.fallback_file_creation(&meta).await?;
            return Err(Status::aborted("file probably corrupted"));
        }

        Ok(Response::new(UploadFileResponse { id: "".to_string() }))
    }

    async fn delete_file_version(
        &self,
        _request: Request<DeleteFileVersionRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_closest_url(
        &self,
        _request: Request<GetClosestUrlRequest>,
    ) -> Result<Response<GetClosestUrlResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}
