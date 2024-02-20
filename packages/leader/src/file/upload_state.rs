use color_eyre::{eyre::eyre, Result};
use futures::lock::Mutex;
use std::sync::Arc;
use tokio::{fs, io::AsyncWriteExt};
use tracing::instrument;

use qcdn_database::{
    Database, Dir, DirUpsert, File, FileUpsert, FileVersion, FileVersionState, NewFileVersion,
};
use qcdn_proto_server::{FilePart, UploadMeta, UploadResponse};
use qcdn_storage::Storage;

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
    dir: Arc<Dir>,
    file: Arc<File>,
    file_version: Arc<Mutex<FileVersion>>,
}

impl FileUploadRequested {
    #[instrument(skip_all)]
    pub async fn try_init(storage: Arc<Storage>, db: Arc<Database>) -> Result<Self> {
        Ok(Self { storage, db })
    }
}

impl FileUploadRequested {
    #[instrument(skip(self))]
    pub async fn got_meta(self, meta: UploadMeta) -> Result<FileUploading> {
        tracing::debug!("Received meta");
        let mut cleanup_dir = None;
        let mut cleanup_file = None;
        let mut cleanup_file_version = None;

        let mut connection = self.db.establish_connection().await?;
        let storage = self.storage.clone();

        let res: Result<FileUploading> = try {
            let dir = DirUpsert {
                name: meta.dir.clone(),
                created_at: None,
            }
            .find_by_name_or_create(&mut connection)
            .await
            .map(Arc::new)?;
            tracing::trace!("Created dir");
            tracing::trace!("{dir:?}");

            cleanup_dir = Some(dir.clone());

            let file_upsert = FileUpsert {
                dir_id: dir.id,
                name: meta.name.clone(),
                file_type: meta.file_type().into(),
                created_at: None,
            };
            let file = file_upsert
                .find_by_name_or_create(&mut connection)
                .await
                .map(Arc::new)?;
            tracing::trace!("Created file");
            tracing::trace!("{file:?}");

            cleanup_file = Some(file.clone());

            let file_version_upsert = NewFileVersion {
                file_id: file.id,
                size: meta.size,
                version: meta.version.clone(),
                state: FileVersionState::Downloading,
                created_at: None,
            };
            let file_version = file_version_upsert
                .create(&mut connection)
                .await
                .map(Mutex::new)
                .map(Arc::new)?;
            tracing::trace!("Created file version");
            tracing::trace!("{file_version:?}");

            cleanup_file_version = Some(file_version.clone());

            let file_handle = storage
                .create_file(
                    &dir.id.to_string(),
                    &file_version.lock().await.id.to_string(),
                )
                .await?;
            tracing::trace!("Created system file");
            tracing::trace!("{file_handle:?}");

            file_handle.set_len(meta.size.try_into()?).await?;

            FileUploading {
                storage: self.storage,
                db: self.db,
                received_bytes: 0,
                meta,
                file_handle,
                dir,
                file,
                file_version,
            }
        };

        if res.is_err() {
            tracing::trace!("Cleaning needed");

            if let Some(dir) = &cleanup_dir {
                tracing::trace!("Cleaning up dir");
                tracing::trace!("{dir:?}");
                dir.delete_if_no_files_exists(&mut connection).await?;
            }
            if let Some(file) = &cleanup_file {
                tracing::trace!("Cleaning up file");
                tracing::trace!("{file:?}");
                file.delete_if_no_versions_exists(&mut connection).await?;
            }
            if let Some(file_version) = &cleanup_file_version {
                tracing::trace!("Cleaning up file_version");
                tracing::trace!("{file_version:?}");
                file_version
                    .lock()
                    .await
                    .unsafe_delete(&mut connection)
                    .await?;
            }
            if let Some((dir, file_version)) = cleanup_dir.zip(cleanup_file_version) {
                tracing::trace!("Cleaning up system file");
                storage
                    .remove_file(
                        &dir.id.to_string(),
                        &file_version.lock().await.id.to_string(),
                    )
                    .await?;
            }
        }

        res
    }
}

impl FileUploading {
    #[instrument(skip_all)]
    pub async fn cleanup(&self) -> Result<()> {
        tracing::trace!("Cleaning requested");
        let mut connection = self.db.establish_connection().await?;

        tracing::trace!("Cleaning up dir");
        tracing::trace!("{:?}", self.dir);
        self.dir.delete_if_no_files_exists(&mut connection).await?;
        tracing::trace!("Cleaning up file");
        tracing::trace!("{:?}", self.file);
        self.file
            .delete_if_no_versions_exists(&mut connection)
            .await?;
        tracing::trace!("Cleaning up file_version");
        tracing::trace!("{:?}", self.file_version);
        self.file_version
            .lock()
            .await
            .unsafe_delete(&mut connection)
            .await?;
        tracing::trace!("Cleaning up system file");
        self.storage
            .remove_file(
                &self.dir.id.to_string(),
                &self.file_version.lock().await.id.to_string(),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip_all, fields(chunk = part.bytes.len()))]
    pub async fn got_part(&mut self, part: FilePart) -> Result<()> {
        tracing::trace!("Incoming bytes {:?}", part.bytes.len());
        let res: Result<()> = try {
            self.received_bytes += i64::try_from(part.bytes.len())?;
            self.file_handle.write_all(&part.bytes).await?;
            self.file_handle.flush().await?;
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
            let mut connection = self.db.establish_connection().await?;
            if self.meta.size != self.received_bytes {
                tracing::debug!("Size check failed");
                return Err(eyre!("file transmission corrupted"));
            }
            tracing::debug!("Updating version state to ready");
            self.file_version
                .lock()
                .await
                .update_state(&mut connection, FileVersionState::Ready)
                .await?;

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
