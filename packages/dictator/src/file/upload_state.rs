use futures::lock::Mutex;
use std::{num::TryFromIntError, sync::Arc};
use tokio::{fs, io::AsyncWriteExt};
use tracing::instrument;

use qcdn_database::{
    Database, Dir, DirUpsert, File, FileUpsert, FileVersion, FileVersionState, NewFileVersion,
};
use qcdn_proto_server::{FilePart, UploadMeta, UploadResponse};
use qcdn_storage::Storage;

use crate::error::{AppError, Result};

pub struct FileUploadRequested {
    storage: Arc<Storage>,
    db: Arc<Database>,
}

pub struct FileUploading {
    storage: Arc<Storage>,
    db: Arc<Database>,
    received_bytes: i64,
    meta: UploadMeta,
    file_handle: fs::File,
    path: String,
    dir: Arc<Dir>,
    file: Arc<File>,
    file_version: Arc<Mutex<FileVersion>>,
}

impl FileUploadRequested {
    #[instrument(skip_all)]
    pub fn new(storage: Arc<Storage>, db: Arc<Database>) -> Self {
        Self { storage, db }
    }
}

impl FileUploadRequested {
    #[instrument(skip(self))]
    pub async fn got_meta(self, meta: UploadMeta) -> Result<FileUploading> {
        tracing::debug!("Received meta");
        let mut cleanup_dir = None;
        let mut cleanup_file = None;
        let mut cleanup_file_version = None;
        let mut cleanup_file_path = None;

        let mut connection = self
            .db
            .establish_connection()
            .await
            .map_err(AppError::from)?;
        let storage = self.storage.clone();

        let res: Result<FileUploading> = try {
            let dir = DirUpsert {
                name: meta.dir.clone(),
                created_at: None,
            }
            .find_by_name_or_create(&mut connection)
            .await
            .map(Arc::new)
            .map_err(AppError::from)?;
            tracing::trace!("Created dir");
            tracing::trace!("{dir:?}");

            cleanup_dir = Some(dir.clone());

            let file_upsert = FileUpsert {
                dir_id: dir.id,
                name: meta.name.clone(),
                media_type: meta.media_type.clone(),
                created_at: None,
            };
            let file = file_upsert
                .find_by_name_or_create(&mut connection)
                .await
                .map(Arc::new)
                .map_err(AppError::from)?;
            tracing::trace!("Created file");
            tracing::trace!("{file:?}");

            cleanup_file = Some(file.clone());

            let file_version_upsert = NewFileVersion {
                file_id: file.id,
                size: meta.size,
                hash: meta.hash.clone(),
                name: meta.version.clone(),
                state: FileVersionState::Downloading,
                created_at: None,
            };
            let file_version = file_version_upsert
                .create(&mut connection)
                .await
                .map(Mutex::new)
                .map(Arc::new)
                .map_err(AppError::from)?;
            tracing::trace!("Created file version");
            tracing::trace!("{:?}", file_version.lock().await);

            cleanup_file_version = Some(file_version.clone());

            let path = file_version
                .lock()
                .await
                .path(&mut connection)
                .await
                .map_err(AppError::from)?
                .to_string();
            tracing::trace!("Storage path {path:?}");

            cleanup_file_path = Some(path.clone());

            let file_handle = storage.create_file(&path).await?;
            tracing::trace!("Created system file");
            tracing::trace!("{file_handle:?}");

            file_handle
                .set_len(meta.size.try_into().map_err(|e: TryFromIntError| {
                    AppError::Other(format!("Size conversion failed {}", e))
                })?)
                .await?;

            FileUploading {
                storage: self.storage,
                db: self.db,
                received_bytes: 0,
                meta,
                file_handle,
                path,
                dir,
                file,
                file_version,
            }
        };

        if res.is_err() {
            tracing::trace!("Cleaning needed");
            if let Some(file_version) = &cleanup_file_version {
                tracing::trace!("Cleaning up file_version");
                tracing::trace!("{file_version:?}");
                file_version
                    .lock()
                    .await
                    .unsafe_delete(&mut connection)
                    .await
                    .map_err(AppError::from)?;
            }

            if let Some(path) = cleanup_file_path {
                tracing::trace!("Cleaning up system file");
                storage.remove_file(&path).await.map_err(AppError::from)?;
            }

            if let Some(file) = &cleanup_file {
                tracing::trace!("Cleaning up file");
                tracing::trace!("{file:?}");
                file.delete_if_no_versions_exists(&mut connection)
                    .await
                    .map_err(AppError::from)?;
            }

            if let Some(dir) = &cleanup_dir {
                tracing::trace!("Cleaning up dir");
                tracing::trace!("{dir:?}");
                dir.delete_if_no_files_exists(&mut connection)
                    .await
                    .map_err(AppError::from)?;
            }
        }

        res
    }
}

impl FileUploading {
    #[instrument(skip_all)]
    pub async fn cleanup(&self) -> Result<()> {
        tracing::debug!("Cleaning requested");

        let mut connection = self.db.establish_connection().await?;

        tracing::trace!("Cleaning up file_version");
        tracing::trace!("{:?}", self.file_version);
        self.file_version
            .lock()
            .await
            .unsafe_delete(&mut connection)
            .await
            .map_err(AppError::from)?;

        tracing::trace!("Cleaning up system file");
        self.storage
            .remove_file(&self.path)
            .await
            .map_err(AppError::from)?;

        tracing::trace!("Cleaning up file");
        tracing::trace!("{:?}", self.file);
        self.file
            .delete_if_no_versions_exists(&mut connection)
            .await
            .map_err(AppError::from)?;

        tracing::trace!("Cleaning up dir");
        tracing::trace!("{:?}", self.dir);
        self.dir
            .delete_if_no_files_exists(&mut connection)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    #[instrument(skip_all, fields(chunk = part.bytes.len()))]
    pub async fn got_part(&mut self, part: FilePart) -> Result<()> {
        tracing::trace!("Incoming bytes {:?}", part.bytes.len());
        let res: Result<()> = try {
            self.received_bytes +=
                i64::try_from(part.bytes.len()).map_err(|e: TryFromIntError| {
                    AppError::Other(format!("Size conversion failed {}", e))
                })?;
            self.file_handle
                .write_all(&part.bytes)
                .await
                .map_err(AppError::from)?;
            self.file_handle.flush().await.map_err(AppError::from)?;
        };

        if res.is_err() {
            self.cleanup().await?;
        }

        res
    }

    #[instrument(skip(self))]
    pub async fn end(self) -> Result<UploadResponse> {
        tracing::debug!("Stream ended");
        let res: Result<UploadResponse> = try {
            let mut connection = self
                .db
                .establish_connection()
                .await
                .map_err(AppError::from)?;
            if self.meta.size != self.received_bytes {
                tracing::debug!("Size check failed");
                return Err(
                    AppError::DataCorruption("file transmission corrupted".to_string()).into(),
                );
            }

            let path = self.storage.get_path(&self.path, false);
            let hash = qcdn_utils::hash::sha256_file(&path)
                .await
                .map_err(AppError::from)?;
            if self.meta.hash != hash {
                tracing::debug!("Hash check failed");
                return Err(
                    AppError::DataCorruption("file transmission corrupted".to_string()).into(),
                );
            }

            tracing::debug!("Updating version state to ready");
            self.file_version
                .lock()
                .await
                .update_state(&mut connection, FileVersionState::Ready)
                .await
                .map_err(AppError::from)?;

            UploadResponse {
                dir_id: self.dir.id,
                file_id: self.file.id,
                file_version_id: self.file_version.lock().await.id,
            }
        };

        if res.is_err() {
            self.cleanup().await?;
        }

        res
    }
}
