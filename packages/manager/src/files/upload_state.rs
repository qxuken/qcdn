use color_eyre::{eyre::eyre, Result};
use tokio::{fs, io::AsyncWriteExt};

use qcdn_database::{
    Database, DatabaseConnection, Dir, DirUpsert, File, FileUpsert, FileVersion, FileVersionState,
    NewFileVersion,
};
use qcdn_proto_server::{FilePart, UploadMeta, UploadResponse};
use qcdn_storage::Storage;

pub struct FileUploadRequested {
    connection: DatabaseConnection,
}

pub struct FileUploading {
    storage: Storage,
    connection: DatabaseConnection,
    received_bytes: i64,
    meta: UploadMeta,
    file_handle: fs::File,
    dir: Dir,
    file: File,
    file_version: FileVersion,
}

impl FileUploadRequested {
    pub async fn init(db: Database) -> Result<Self> {
        let connection = db.establish_connection().await?;
        Ok(Self { connection })
    }
}

impl FileUploadRequested {
    pub async fn got_meta(mut self, storage: Storage, meta: UploadMeta) -> Result<FileUploading> {
        let dir = DirUpsert {
            name: meta.dir.clone(),
            created_at: None,
        }
        .find_by_name_or_create(&mut self.connection)
        .await?;

        let file_upsert = FileUpsert {
            dir_id: dir.id,
            name: meta.name.clone(),
            file_type: meta.file_type().into(),
            created_at: None,
        };
        let file = match file_upsert
            .find_by_name_or_create(&mut self.connection)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                dir.delete_if_no_files_exists(&mut self.connection).await?;
                return Err(eyre!(e.to_string()));
            }
        };

        let file_version_upsert = NewFileVersion {
            file_id: file.id,
            size: meta.size,
            version: meta.version.clone(),
            state: FileVersionState::Downloading,
            created_at: None,
        };
        let file_version = match file_version_upsert.create(&mut self.connection).await {
            Ok(id) => id,
            Err(e) => {
                dir.delete_if_no_files_exists(&mut self.connection).await?;
                file.delete_if_no_versions_exists(&mut self.connection)
                    .await?;
                return Err(eyre!(e.to_string()));
            }
        };

        let file_handle = match storage
            .create_file(&dir.id.to_string(), &file_version.id.to_string())
            .await
        {
            Ok(id) => id,
            Err(e) => {
                dir.delete_if_no_files_exists(&mut self.connection).await?;
                file.delete_if_no_versions_exists(&mut self.connection)
                    .await?;
                file_version.unsafe_delete(&mut self.connection).await?;
                return Err(eyre!(e.to_string()));
            }
        };

        if let Err(e) = file_handle.set_len(meta.size.try_into()?).await {
            storage
                .remove_file(&dir.id.to_string(), &file_version.id.to_string())
                .await?;

            dir.delete_if_no_files_exists(&mut self.connection).await?;
            file.delete_if_no_versions_exists(&mut self.connection)
                .await?;
            file_version.unsafe_delete(&mut self.connection).await?;

            return Err(eyre!(e.to_string()));
        }

        Ok(FileUploading {
            storage,
            connection: self.connection,
            received_bytes: 0,
            meta,
            file_handle,
            dir,
            file,
            file_version,
        })
    }
}

impl FileUploading {
    pub async fn cleanup(&mut self) -> Result<()> {
        self.storage
            .remove_file(&self.dir.id.to_string(), &self.file_version.id.to_string())
            .await?;

        self.dir
            .delete_if_no_files_exists(&mut self.connection)
            .await?;

        self.file
            .delete_if_no_versions_exists(&mut self.connection)
            .await?;

        self.file_version
            .unsafe_delete(&mut self.connection)
            .await?;

        Ok(())
    }

    pub async fn got_part(&mut self, part: FilePart) -> Result<()> {
        match i64::try_from(part.bytes.len()) {
            Ok(b) => self.received_bytes += b,
            Err(e) => {
                self.cleanup().await?;
                return Err(eyre!(e.to_string()));
            }
        };
        if let Err(e) = self.file_handle.write(&part.bytes).await {
            self.cleanup().await?;
            return Err(eyre!(e.to_string()));
        }
        Ok(())
    }

    pub async fn end(mut self) -> Result<UploadResponse> {
        if self.meta.size != self.received_bytes {
            self.cleanup().await?;
            return Err(eyre!("file transmission corrupted"));
        }
        if let Err(e) = self
            .file_version
            .update_state(&mut self.connection, FileVersionState::Ready)
            .await
        {
            self.cleanup().await?;
            return Err(eyre!(e.to_string()));
        }
        Ok(UploadResponse {
            dir_id: self.dir.id,
            file_id: self.file.id,
            file_version_id: self.file_version.id,
        })
    }
}
