use std::{pin::Pin, sync::Arc};
use tokio_stream::{Stream, StreamExt};
use tokio_util::io::ReaderStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::instrument;

use qcdn_database::{Database, Dir, File, FileVersion, FileVersionTagUpsert, FileVersionWithTags};
use qcdn_proto_server::{
    qcdn_file_queries_server::QcdnFileQueries, qcdn_file_updates_server::QcdnFileUpdates,
    upload_request, DeleteFileVersionRequest, DownloadRequest, FilePart, GetDirRequest,
    GetDirResponse, GetDirsResponse, GetFileRequest, GetFileResponse, GetFileVersionRequest,
    GetFileVersionResponse, GetFileVersionsRequest, GetFileVersionsResponse, GetFilesRequest,
    GetFilesResponse, TagVersionRequest, UploadRequest, UploadResponse,
};
use qcdn_storage::Storage;

use self::upload_state::FileUploadRequested;

use crate::error::Result;

mod upload_state;

#[derive(Debug, Clone)]
pub struct FileService {
    storage: Arc<Storage>,
    db: Arc<Database>,
}

impl FileService {
    pub fn new(storage: Arc<Storage>, db: Arc<Database>) -> Self {
        Self { storage, db }
    }
}

#[tonic::async_trait]
impl QcdnFileQueries for FileService {
    #[instrument(skip(self))]
    async fn get_dirs(&self, _request: Request<()>) -> Result<Response<GetDirsResponse>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let items = Dir::get_all(&mut connection)
            .await?
            .into_iter()
            .map(|item| item.into())
            .collect();
        tracing::trace!("{items:?}");

        Ok(Response::new(GetDirsResponse { items }))
    }

    #[instrument(skip(self))]
    async fn get_dir(
        &self,
        request: Request<GetDirRequest>,
    ) -> Result<Response<GetDirResponse>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let id = request.into_inner().id;
        let item = Dir::find_by_id(&mut connection, &id).await?.into();
        tracing::trace!("{item:?}");

        Ok(Response::new(item))
    }

    #[instrument(skip(self))]
    async fn get_files(
        &self,
        request: Request<GetFilesRequest>,
    ) -> Result<Response<GetFilesResponse>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let dir_id = request.into_inner().dir_id;

        let items = File::find_all_by_dir(&mut connection, &dir_id)
            .await?
            .into_iter()
            .map(|item| item.into())
            .collect();
        tracing::trace!("{items:?}");

        Ok(Response::new(GetFilesResponse { items }))
    }

    #[instrument(skip(self))]
    async fn get_file(
        &self,
        request: Request<GetFileRequest>,
    ) -> Result<Response<GetFileResponse>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let id = request.into_inner().id;
        let item = File::find_by_id(&mut connection, &id).await?.into();
        tracing::trace!("{item:?}");

        Ok(Response::new(item))
    }

    #[instrument(skip(self))]
    async fn get_file_versions(
        &self,
        request: Request<GetFileVersionsRequest>,
    ) -> Result<Response<GetFileVersionsResponse>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let file_id = request.into_inner().file_id;

        let items = FileVersionWithTags::find_by_file_id(&mut connection, &file_id)
            .await?
            .into_iter()
            .map(|item| item.into())
            .collect();
        tracing::trace!("{items:?}");

        Ok(Response::new(GetFileVersionsResponse { items }))
    }

    #[instrument(skip(self))]
    async fn get_file_version(
        &self,
        request: Request<GetFileVersionRequest>,
    ) -> Result<Response<GetFileVersionResponse>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let id = request.into_inner().id;

        let item = FileVersionWithTags::find_by_id(&mut connection, &id)
            .await?
            .into();
        tracing::trace!("{item:?}");

        Ok(Response::new(item))
    }

    type DownloadStream = Pin<Box<dyn Stream<Item = Result<FilePart, Status>> + Send>>;

    #[instrument(skip(self))]
    async fn download(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadStream>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let file_version_id = request.into_inner().file_version_id;

        let path = FileVersion::find_by_id(&mut connection, &file_version_id)
            .await?
            .path(&mut connection)
            .await?
            .to_string();
        tracing::trace!("Storage path {path:?}");

        let file = self
            .storage
            .open_file(&path)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let stream = ReaderStream::new(file).map(|frame| {
            frame
                .map(|bytes| FilePart {
                    bytes: bytes.into(),
                })
                .map_err(|e| Status::internal(e.to_string()))
        });

        tracing::debug!("Sending file stream");

        Ok(Response::new(Box::pin(stream)))
    }
}

#[tonic::async_trait]
impl QcdnFileUpdates for FileService {
    #[instrument(skip(self))]
    async fn upload(
        &self,
        request: Request<Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
        tracing::debug!("Got request");
        let mut in_stream = request.into_inner();

        let state = FileUploadRequested::new(self.storage.clone(), self.db.clone());

        let mut state = if let Some(result) = in_stream
            .next()
            .await
            .and_then(|r| r.map(|r| r.request).transpose())
            .transpose()?
        {
            match result {
                upload_request::Request::Meta(meta) => state.got_meta(meta).await?,
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
                    state.cleanup().await?;
                    return Err(Status::aborted(
                        "UploadFileMeta message cannot be sent twice",
                    ));
                }
                upload_request::Request::Part(part) => state.got_part(part).await?,
            };
        }

        let res = state.end().await?;

        Ok(Response::new(res))
    }

    #[instrument(skip(self))]
    async fn tag_version(
        &self,
        request: Request<TagVersionRequest>,
    ) -> Result<Response<()>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let fvt: FileVersionTagUpsert = request.into_inner().into();
        tracing::trace!("{fvt:?}");
        fvt.create_or_move(&mut connection).await?;

        Ok(Response::new(()))
    }

    #[instrument(skip(self))]
    async fn delete_file_version(
        &self,
        request: Request<DeleteFileVersionRequest>,
    ) -> Result<Response<()>, Status> {
        tracing::debug!("Got request");
        let mut connection = self.db.establish_connection().await?;

        let id = request.into_inner().id;

        let mut fv = FileVersion::find_by_id(&mut connection, &id).await?;
        tracing::trace!("{fv:?}");

        fv.delete(&mut connection).await?;

        Ok(Response::new(()))
    }
}
