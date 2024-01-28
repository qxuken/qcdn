use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

use crate::{
    database::files::{
        file_search::{FileSearch, FileVersionSearch},
        file_version_record::FileVersionRecord,
    },
    entities::files::FileUploadRequested,
    grpc::{
        qcdn_files_server::QcdnFiles, upload_file_request, DeleteFileVersionRequest,
        GetFileVersionsRequest, GetFileVersionsResponse, GetFilesResponse, UploadFileRequest,
        UploadFileResponse,
    },
    AppState,
};

#[derive(Debug, Clone)]
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
        tracing::info!("Get files request recieved");
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let items = FileSearch::get_all(&mut connection)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .into_iter()
            .map(|item| item.into())
            .collect();

        Ok(Response::new(GetFilesResponse { items }))
    }

    async fn get_file_versions(
        &self,
        request: Request<GetFileVersionsRequest>,
    ) -> Result<Response<GetFileVersionsResponse>, Status> {
        tracing::info!("Get file versions request recieved");
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let file_id = request.into_inner().file_id;

        let items = FileVersionSearch::find_by_file_id(&mut connection, &file_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .into_iter()
            .map(|item| item.into())
            .collect();

        Ok(Response::new(GetFileVersionsResponse { items }))
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
        request: Request<DeleteFileVersionRequest>,
    ) -> Result<Response<()>, Status> {
        tracing::info!("Delete file version request recieved");
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let file_version_id = request.into_inner().id;
        let file_version_id = uuid::Uuid::parse_str(&file_version_id)
            .map_err(|e| Status::invalid_argument(format!("id is not valid uuid {e:?}")))?;

        match FileVersionRecord::find_by_id(&mut connection, file_version_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
        {
            Some(fv) => fv
                .delete(&mut connection)
                .await
                .map_err(|e| Status::internal(e.to_string())),
            None => Err(Status::not_found("File version not found")),
        }
        .map(Response::new)
    }
}
