use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqliteConnection};

use crate::{
    database::utils,
    grpc::{self, sync_message, DeletedVersion, UploadedVersion, VersionTagged},
};

use super::records::file_version_record::FileVersionState;

#[derive(Debug)]
pub enum FileSyncAction {
    UploadedVersion { dir_id: String, file_id: String },
    VersionTagged { tag: String },
    DeletedVersion,
}

#[derive(Debug)]
pub struct FileSync {
    pub action: FileSyncAction,
    pub file_version_id: String,
    pub timestamp: DateTime<Utc>,
}

impl From<FileSync> for grpc::SyncMessage {
    fn from(value: FileSync) -> Self {
        let file_version_id = value.file_version_id;
        let message_type = match value.action {
            FileSyncAction::UploadedVersion { dir_id, file_id } => {
                sync_message::MessageType::Uploaded(UploadedVersion {
                    dir_id,
                    file_id,
                    file_version_id,
                })
            }
            FileSyncAction::VersionTagged { tag } => {
                sync_message::MessageType::Tagged(VersionTagged {
                    tag,
                    file_version_id,
                })
            }
            FileSyncAction::DeletedVersion => {
                sync_message::MessageType::Deleted(DeletedVersion { file_version_id })
            }
        };
        let ts: SystemTime = value.timestamp.into();
        Self {
            message_type: Some(message_type),
            timestamp: Some(ts.into()),
        }
    }
}

impl FileSync {
    pub async fn uploaded_from_ts(
        connection: &mut SqliteConnection,
        ts: &DateTime<Utc>,
    ) -> Result<Vec<Self>> {
        let ts = ts.timestamp();

        sqlx::query(
            r#"
                SELECT
                    fv.id file_version_id,
                    f.id file_id,
                    f.dir_id,
                    fv.created_at
                FROM
                    file_version fv
                    INNER JOIN file f ON f.id = fv.file_id
                WHERE
                    fv.state = ?
                    AND fv.created_at > ?
            "#,
        )
        .bind(FileVersionState::Ready)
        .bind(ts)
        .fetch_all(connection)
        .await?
        .into_iter()
        .map(|row| {
            let timestamp = utils::parse_timestamp(&row, "created_at")?;
            let dir_id = row.try_get("dir_id")?;
            let file_id = row.try_get("file_id")?;
            let file_version_id = row.try_get("file_version_id")?;

            Ok(FileSync {
                action: FileSyncAction::UploadedVersion { dir_id, file_id },
                file_version_id,
                timestamp,
            })
        })
        .collect::<Result<Vec<Self>>>()
    }

    pub async fn deleted_from_ts(
        connection: &mut SqliteConnection,
        ts: &DateTime<Utc>,
    ) -> Result<Vec<Self>> {
        let ts = ts.timestamp();

        sqlx::query(
            r#"
                SELECT
                    fv.id file_version_id,
                    fv.deleted_at
                FROM file_version fv
                WHERE fv.deleted_at > ?
            "#,
        )
        .bind(ts)
        .fetch_all(connection)
        .await?
        .into_iter()
        .map(|row| {
            let timestamp = utils::parse_timestamp(&row, "deleted_at")?;
            let file_version_id = row.try_get("file_version_id")?;

            Ok(FileSync {
                action: FileSyncAction::DeletedVersion,
                file_version_id,
                timestamp,
            })
        })
        .collect::<Result<Vec<Self>>>()
    }

    pub async fn tagged_from_ts(
        connection: &mut SqliteConnection,
        ts: &DateTime<Utc>,
    ) -> Result<Vec<Self>> {
        let ts = ts.timestamp();

        sqlx::query(
            r#"
                SELECT
                    fvt.file_version_id,
                    fvt.name,
                    fvt.activated_at
                FROM file_version_tag fvt
                WHERE fvt.activated_at > ?
            "#,
        )
        .bind(ts)
        .fetch_all(connection)
        .await?
        .into_iter()
        .map(|row| {
            let timestamp = utils::parse_timestamp(&row, "activated_at")?;
            let file_version_id = row.try_get("file_version_id")?;
            let tag = row.try_get("name")?;

            Ok(FileSync {
                action: FileSyncAction::VersionTagged { tag },
                file_version_id,
                timestamp,
            })
        })
        .collect::<Result<Vec<Self>>>()
    }
}
