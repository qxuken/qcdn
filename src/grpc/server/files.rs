use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use crate::{
    entities::files::FileUploadRequested,
    grpc::{
        qcdn_files_server::QcdnFiles, upload_file_request, DeleteFileVersionRequest,
        GetFileVersionsRequest, GetFileVersionsResponse, GetFilesResponse, UploadFileRequest,
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

#[tonic::async_trait]
impl QcdnFiles for FilesService {
    async fn get_files(&self, _request: Request<()>) -> Result<Response<GetFilesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_file_versions(
        &self,
        _request: Request<GetFileVersionsRequest>,
    ) -> Result<Response<GetFileVersionsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn upload_file(
        &self,
        request: Request<Streaming<UploadFileRequest>>,
    ) -> Result<Response<UploadFileResponse>, Status> {
        tracing::info!("Upload file stream initiated");
        let mut in_stream = request.into_inner();

        let state = FileUploadRequested::init(self.app_state.db.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut state = if let Some(result) = in_stream
            .next()
            .await
            .and_then(|r| r.map(|r| r.request).transpose())
            .transpose()?
        {
            match result {
                upload_file_request::Request::Meta(meta) => state
                    .got_meta(self.app_state.storage.clone(), meta)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?,
                _ => {
                    return Err(Status::failed_precondition(
                        "UploadFileMeta should be first message",
                    ));
                }
            }
        } else {
            return Err(Status::failed_precondition("Message not received"));
        };

        while let Some(result) = in_stream
            .next()
            .await
            .and_then(|r| r.map(|r| r.request).transpose())
            .transpose()?
        {
            match result {
                upload_file_request::Request::Meta(_) => {
                    state
                        .cleanup()
                        .await
                        .map_err(|e| Status::internal(e.to_string()))?;
                    return Err(Status::aborted(
                        "UploadFileMeta message cannot be sent twice",
                    ));
                }
                upload_file_request::Request::Part(part) => state
                    .got_part(part)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?,
            };
        }

        tracing::info!("Upload file stream ended");
        let (file_record_id, file_version_record_id) = state
            .end()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UploadFileResponse {
            file_id: file_record_id.to_string(),
            file_version_id: file_version_record_id.to_string(),
        }))
    }

    async fn delete_file_version(
        &self,
        _request: Request<DeleteFileVersionRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}
