use anyhow::{bail, Result};
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

use crate::{
    database::{
        files::{
            file_record::FileRecord,
            file_version_record::{FileVersionRecord, FileVersionState},
        },
        Database,
    },
    grpc::{UploadFileMeta, UploadFilePart},
    DatabasePoolConnection, Storage,
};

pub struct FileUploadRequested {
    connection: DatabasePoolConnection,
}

pub struct FileUploading {
    storage: Storage,
    connection: DatabasePoolConnection,
    received_bytes: u64,
    meta: UploadFileMeta,
    file: fs::File,
    file_record: FileRecord,
    file_version_record: FileVersionRecord,
}

impl FileUploadRequested {
    pub async fn init(db: Database) -> Result<Self> {
        let connection = db.connect().await?;
        Ok(Self { connection })
    }
}

impl FileUploadRequested {
    pub async fn got_meta(
        mut self,
        storage: Storage,
        meta: UploadFileMeta,
    ) -> Result<FileUploading> {
        let file_record = FileRecord::find_or_create(
            &mut self.connection,
            &meta.dir,
            &meta.name,
            meta.file_type().into(),
        )
        .await?;

        let file_version_record = match FileVersionRecord::create(
            &mut self.connection,
            &file_record.id,
            &meta.version,
            meta.size,
            FileVersionState::Downloading,
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                file_record
                    .delete_if_no_versions_exists(&mut self.connection)
                    .await?;
                bail!(e)
            }
        };

        let file = match storage
            .create_file(&meta.dir, &file_version_record.id.to_string())
            .await
        {
            Ok(id) => id,
            Err(e) => {
                file_record
                    .delete_if_no_versions_exists(&mut self.connection)
                    .await?;
                file_version_record
                    .force_delete(&mut self.connection)
                    .await?;
                bail!(e)
            }
        };

        if let Err(e) = file.set_len(meta.size).await {
            file_record
                .delete_if_no_versions_exists(&mut self.connection)
                .await?;
            file_version_record
                .force_delete(&mut self.connection)
                .await?;
            storage.remove_file(&meta.dir, &meta.name).await?;
            bail!(e)
        }

        Ok(FileUploading {
            storage,
            connection: self.connection,
            received_bytes: 0,
            meta,
            file,
            file_record,
            file_version_record,
        })
    }
}

impl FileUploading {
    pub async fn cleanup(&mut self) -> Result<()> {
        self.storage
            .remove_file(&self.meta.dir, &self.meta.name)
            .await?;

        self.file_record
            .delete_if_no_versions_exists(&mut self.connection)
            .await?;

        self.file_version_record
            .force_delete(&mut self.connection)
            .await?;

        Ok(())
    }

    pub async fn got_part(&mut self, part: UploadFilePart) -> Result<()> {
        self.received_bytes += part.bytes.len() as u64;
        if let Err(e) = self.file.write(&part.bytes).await {
            self.cleanup().await?;
            bail!(e)
        }
        Ok(())
    }

    pub async fn end(mut self) -> Result<(Uuid, Uuid)> {
        if self.meta.size != self.received_bytes {
            self.cleanup().await?;
            bail!("file transmission corrupted")
        }
        if let Err(e) = self
            .file_version_record
            .update_state(&mut self.connection, FileVersionState::Ready)
            .await
        {
            self.cleanup().await?;
            bail!(e)
        }
        if self.meta.is_latest {
            if let Err(e) = self
                .file_version_record
                .make_latest(&mut self.connection)
                .await
            {
                self.cleanup().await?;
                bail!(e)
            }
        }
        Ok((self.file_record.id, self.file_version_record.id))
    }
}
