use std::{pin::Pin, sync::Arc, time::SystemTime};

use async_channel::Sender;
use tokio_stream::{Stream, StreamExt};
use tokio_util::io::ReaderStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::instrument;

use crate::{
    database::files::{
        records::{
            file_version_record::FileVersionRecord, file_version_tag_record::FileVersionTagRecord,
        },
        search::{
            dir_search::DirSearch, file_search::FileSearch, file_version_search::FileVersionSearch,
        },
    },
    entities::files::upload_state::FileUploadRequested,
    grpc::{
        qcdn_files_server::QcdnFiles, sync_message::MessageType, upload_request,
        DeleteFileVersionRequest, DeletedVersion, DownloadRequest, FilePart, GetDirRequest,
        GetDirResponse, GetDirsResponse, GetFileRequest, GetFileResponse, GetFileVersionRequest,
        GetFileVersionResponse, GetFileVersionsRequest, GetFileVersionsResponse, GetFilesRequest,
        GetFilesResponse, SyncMessage, TagVersionRequest, UploadRequest, UploadResponse,
        VersionTagged,
    },
    AppState,
};

#[derive(Debug, Clone)]
pub struct FilesService {
    app_state: Arc<AppState>,
    sync: Sender<SyncMessage>,
}

impl FilesService {
    pub fn new(app_state: Arc<AppState>, sync: Sender<SyncMessage>) -> Self {
        Self { app_state, sync }
    }
}

#[tonic::async_trait]
impl QcdnFiles for FilesService {
    #[instrument]
    async fn get_dirs(&self, _request: Request<()>) -> Result<Response<GetDirsResponse>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let items = DirSearch::get_all(&mut connection)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .into_iter()
            .map(|item| item.into())
            .collect();

        Ok(Response::new(GetDirsResponse { items }))
    }

    #[instrument]
    async fn get_dir(
        &self,
        request: Request<GetDirRequest>,
    ) -> Result<Response<GetDirResponse>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let id = request.into_inner().id;
        let id = uuid::Uuid::parse_str(&id)
            .map_err(|e| Status::invalid_argument(format!("id is not valid uuid {e:?}")))?;

        let item = DirSearch::find_by_id(&mut connection, &id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or(Status::not_found("Dir not found"))?
            .into();

        Ok(Response::new(item))
    }

    #[instrument]
    async fn get_files(
        &self,
        request: Request<GetFilesRequest>,
    ) -> Result<Response<GetFilesResponse>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let items = match request.into_inner().dir_id {
            Some(dir_id) => {
                let dir_id = uuid::Uuid::parse_str(&dir_id).map_err(|e| {
                    Status::invalid_argument(format!("dir_id is not valid uuid {e:?}"))
                })?;
                FileSearch::find_by_dir_id(&mut connection, &dir_id).await
            }
            None => FileSearch::get_all(&mut connection).await,
        }
        .map_err(|e| Status::internal(e.to_string()))?
        .into_iter()
        .map(|item| item.into())
        .collect();

        Ok(Response::new(GetFilesResponse { items }))
    }

    #[instrument]
    async fn get_file(
        &self,
        request: Request<GetFileRequest>,
    ) -> Result<Response<GetFileResponse>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let id = request.into_inner().id;
        let id = uuid::Uuid::parse_str(&id)
            .map_err(|e| Status::invalid_argument(format!("id is not valid uuid {e:?}")))?;

        let item = FileSearch::find_by_id(&mut connection, &id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or(Status::not_found("File not found"))?
            .into();

        Ok(Response::new(item))
    }

    #[instrument]
    async fn get_file_versions(
        &self,
        request: Request<GetFileVersionsRequest>,
    ) -> Result<Response<GetFileVersionsResponse>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let file_id = request.into_inner().file_id;
        let file_id = uuid::Uuid::parse_str(&file_id)
            .map_err(|e| Status::invalid_argument(format!("file_id is not valid uuid {e:?}")))?;

        let items = FileVersionSearch::find_by_file_id(&mut connection, &file_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .into_iter()
            .map(|item| item.into())
            .collect();

        Ok(Response::new(GetFileVersionsResponse { items }))
    }

    #[instrument]
    async fn get_file_version(
        &self,
        request: Request<GetFileVersionRequest>,
    ) -> Result<Response<GetFileVersionResponse>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let id = request.into_inner().id;
        let id = uuid::Uuid::parse_str(&id)
            .map_err(|e| Status::invalid_argument(format!("id is not valid uuid {e:?}")))?;

        let item = FileVersionSearch::find_by_id(&mut connection, &id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or(Status::not_found("FileVersionSearch not found"))?
            .into();

        Ok(Response::new(item))
    }

    #[instrument]
    async fn upload(
        &self,
        request: Request<Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
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
                upload_request::Request::Meta(meta) => state
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
                upload_request::Request::Meta(_) => {
                    state
                        .cleanup()
                        .await
                        .map_err(|e| Status::internal(e.to_string()))?;
                    return Err(Status::aborted(
                        "UploadFileMeta message cannot be sent twice",
                    ));
                }
                upload_request::Request::Part(part) => state
                    .got_part(part)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?,
            };
        }

        let (dir_id, file_id, file_version_id) = state
            .end(self.sync.clone())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UploadResponse {
            dir_id: dir_id.to_string(),
            file_id: file_id.to_string(),
            file_version_id: file_version_id.to_string(),
        }))
    }

    type DownloadStream = Pin<Box<dyn Stream<Item = Result<FilePart, Status>> + Send>>;

    #[instrument]
    async fn download(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadStream>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let file_version_id = request.into_inner().file_version_id;
        let file_version_id = uuid::Uuid::parse_str(&file_version_id).map_err(|e| {
            Status::invalid_argument(format!("file_version_id is not valid uuid {e:?}"))
        })?;
        let (dir_id, file_version_id) =
            FileVersionRecord::find_by_id(&mut connection, &file_version_id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?
                .ok_or(Status::not_found("File version not found"))?
                .path(&mut connection)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

        let file = self
            .app_state
            .storage
            .open_file(&dir_id, &file_version_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let stream = ReaderStream::new(file).map(|frame| {
            frame
                .map(|bytes| FilePart {
                    bytes: bytes.into(),
                })
                .map_err(|e| Status::internal(e.to_string()))
        });

        Ok(Response::new(Box::pin(stream)))
    }

    #[instrument]
    async fn tag_version(
        &self,
        request: Request<TagVersionRequest>,
    ) -> Result<Response<()>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let request = request.into_inner();
        let file_version_id = request.file_version_id;
        let file_version_id = uuid::Uuid::parse_str(&file_version_id).map_err(|e| {
            Status::invalid_argument(format!("file_version_id is not valid uuid {e:?}"))
        })?;
        let tag = request.tag;

        let t = FileVersionTagRecord::create_or_move(&mut connection, &file_version_id, &tag, None)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let ts: SystemTime = t.activated_at.into();
        if let Err(e) = self
            .sync
            .clone()
            .send(SyncMessage {
                message_type: Some(MessageType::Tagged(VersionTagged {
                    file_version_id: t.file_version_id.to_string(),
                    tag: t.name,
                })),
                timestamp: Some(ts.into()),
            })
            .await
        {
            tracing::error!("{e:?}");
        };

        Ok(Response::new(()))
    }

    #[instrument]
    async fn delete_file_version(
        &self,
        request: Request<DeleteFileVersionRequest>,
    ) -> Result<Response<()>, Status> {
        let mut connection = self
            .app_state
            .db
            .connect()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let id = request.into_inner().id;
        let id = uuid::Uuid::parse_str(&id)
            .map_err(|e| Status::invalid_argument(format!("id is not valid uuid {e:?}")))?;

        let mut fv = FileVersionRecord::find_by_id(&mut connection, &id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or(Status::not_found("File version not found"))?;

        fv.delete(&mut connection, None)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        if let Err(e) = self
            .sync
            .clone()
            .send(SyncMessage {
                message_type: Some(MessageType::Deleted(DeletedVersion {
                    file_version_id: fv.id.to_string(),
                })),
                timestamp: fv
                    .deleted_at
                    .map(|ts| ts.into())
                    .map(|ts: SystemTime| ts.into()),
            })
            .await
        {
            tracing::error!("{e:?}");
        };

        Ok(Response::new(()))
    }
}
