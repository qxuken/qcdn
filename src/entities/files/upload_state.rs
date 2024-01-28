use std::time::SystemTime;

use anyhow::{bail, Result};
use async_channel::Sender;
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

use crate::{
    database::{
        files::records::{
            dir_record::DirRecord,
            file_record::FileRecord,
            file_version_record::{FileVersionRecord, FileVersionState},
        },
        Database,
    },
    grpc::{sync_message::MessageType, FilePart, SyncMessage, UploadMeta, UploadedVersion},
    DatabasePoolConnection, Storage,
};

pub struct FileUploadRequested {
    connection: DatabasePoolConnection,
}

pub struct FileUploading {
    storage: Storage,
    connection: DatabasePoolConnection,
    received_bytes: u64,
    meta: UploadMeta,
    file: fs::File,
    dir_record: DirRecord,
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
    pub async fn got_meta(mut self, storage: Storage, meta: UploadMeta) -> Result<FileUploading> {
        let dir_record =
            DirRecord::find_by_name_or_create(&mut self.connection, &meta.dir, None).await?;

        let file_record = match FileRecord::find_by_name_or_create(
            &mut self.connection,
            &dir_record.id,
            &meta.name,
            meta.file_type().into(),
            None,
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                dir_record
                    .delete_if_no_files_exists(&mut self.connection)
                    .await?;
                bail!(e)
            }
        };

        let file_version_record = match FileVersionRecord::create(
            &mut self.connection,
            &file_record.id,
            &meta.version,
            meta.size,
            FileVersionState::Downloading,
            None,
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                dir_record
                    .delete_if_no_files_exists(&mut self.connection)
                    .await?;
                file_record
                    .delete_if_no_versions_exists(&mut self.connection)
                    .await?;
                bail!(e)
            }
        };

        let file = match storage
            .create_file(
                &dir_record.id.to_string(),
                &file_version_record.id.to_string(),
            )
            .await
        {
            Ok(id) => id,
            Err(e) => {
                dir_record
                    .delete_if_no_files_exists(&mut self.connection)
                    .await?;
                file_record
                    .delete_if_no_versions_exists(&mut self.connection)
                    .await?;
                file_version_record
                    .unsafe_delete(&mut self.connection)
                    .await?;
                bail!(e)
            }
        };

        if let Err(e) = file.set_len(meta.size).await {
            storage
                .remove_file(
                    &dir_record.id.to_string(),
                    &file_version_record.id.to_string(),
                )
                .await?;

            dir_record
                .delete_if_no_files_exists(&mut self.connection)
                .await?;
            file_record
                .delete_if_no_versions_exists(&mut self.connection)
                .await?;
            file_version_record
                .unsafe_delete(&mut self.connection)
                .await?;

            bail!(e)
        }

        Ok(FileUploading {
            storage,
            connection: self.connection,
            received_bytes: 0,
            meta,
            file,
            dir_record,
            file_record,
            file_version_record,
        })
    }
}

impl FileUploading {
    pub async fn cleanup(&mut self) -> Result<()> {
        self.storage
            .remove_file(
                &self.dir_record.id.to_string(),
                &self.file_version_record.id.to_string(),
            )
            .await?;

        self.dir_record
            .delete_if_no_files_exists(&mut self.connection)
            .await?;

        self.file_record
            .delete_if_no_versions_exists(&mut self.connection)
            .await?;

        self.file_version_record
            .unsafe_delete(&mut self.connection)
            .await?;

        Ok(())
    }

    pub async fn got_part(&mut self, part: FilePart) -> Result<()> {
        self.received_bytes += part.bytes.len() as u64;
        if let Err(e) = self.file.write(&part.bytes).await {
            self.cleanup().await?;
            bail!(e)
        }
        Ok(())
    }

    pub async fn end(mut self, sync: Sender<SyncMessage>) -> Result<(Uuid, Uuid, Uuid)> {
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
        let ts: SystemTime = self.file_version_record.created_at.into();
        if let Err(e) = sync
            .send(SyncMessage {
                message_type: Some(MessageType::Uploaded(UploadedVersion {
                    dir_id: self.dir_record.id.to_string(),
                    file_id: self.file_record.id.to_string(),
                    file_version_id: self.file_version_record.id.to_string(),
                })),
                timestamp: Some(ts.into()),
            })
            .await
        {
            tracing::error!("{e:?}");
        };
        Ok((
            self.dir_record.id,
            self.file_record.id,
            self.file_version_record.id,
        ))
    }
}
