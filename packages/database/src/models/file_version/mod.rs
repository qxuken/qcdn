use chrono::{NaiveDateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::Connection;
use tracing::instrument;

pub use file_version_path_parts::FileVersionPathParts;
pub use file_version_state::FileVersionState;
pub use file_version_with_tags::FileVersionWithTags;
pub use new_file_version::NewFileVersion;

use crate::{DatabaseConnection, DatabaseError};

mod file_version_path_parts;
mod file_version_state;
mod file_version_with_tags;
mod new_file_version;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileVersion {
    pub id: i64,
    pub file_id: i64,
    pub size: i64,
    pub name: String,
    pub state: FileVersionState,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl FileVersion {
    #[instrument(skip(connection))]
    pub async fn find_by_file_id(
        connection: &mut DatabaseConnection,
        file_id: &i64,
    ) -> Result<Vec<Self>, DatabaseError> {
        let items = sqlx::query_as!(
            Self,
            "SELECT * FROM file_version WHERE file_id = ?",
            file_id,
        )
        .fetch_all(connection)
        .await?;

        Ok(items)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_id(
        connection: &mut DatabaseConnection,
        id: &i64,
    ) -> Result<Self, DatabaseError> {
        let item = sqlx::query_as!(Self, "SELECT * FROM file_version WHERE id = ?", id)
            .fetch_one(connection)
            .await?;

        Ok(item)
    }

    #[instrument(skip(connection))]
    pub async fn find_ready(
        connection: &mut DatabaseConnection,
        file_id: &i64,
        name: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        let item = sqlx::query_as!(
            Self,
            "SELECT * FROM file_version WHERE file_id = ? AND name = ? AND state = ? AND deleted_at IS NULL",
            file_id,
            name,
            FileVersionState::Ready,
        )
        .fetch_optional(connection)
        .await?;

        Ok(item)
    }
}

impl FileVersion {
    #[instrument(skip(connection))]
    pub async fn path(
        &self,
        connection: &mut DatabaseConnection,
    ) -> Result<FileVersionPathParts, DatabaseError> {
        let item = sqlx::query!(
            r#"
                SELECT
                    f.dir_id dir,
                    f.id file,
                    fv.id version
                FROM
                    file_version fv
                    INNER JOIN file f ON f.id = fv.file_id
                WHERE fv.id = ?1"#,
            self.id,
        )
        .fetch_one(connection)
        .await?;

        Ok(FileVersionPathParts {
            dir: item.dir.to_string(),
            file: item.file.to_string(),
            version: item.version.to_string(),
        })
    }

    #[instrument(skip(connection))]
    pub async fn update_state(
        &mut self,
        connection: &mut DatabaseConnection,
        state: FileVersionState,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "UPDATE file_version SET state = ?2 WHERE id = ?1",
            self.id,
            state,
        )
        .execute(connection)
        .await?;

        self.state = state;

        Ok(())
    }

    #[instrument(skip(connection))]
    pub async fn delete(
        &mut self,
        connection: &mut DatabaseConnection,
    ) -> Result<(), DatabaseError> {
        let deleted_at = Utc::now().naive_utc();

        sqlx::query!(
            "UPDATE file_version SET deleted_at = ?2 WHERE id = ?1",
            self.id,
            deleted_at,
        )
        .execute(connection)
        .await?;

        self.deleted_at = Some(deleted_at);

        Ok(())
    }

    #[instrument(skip(connection))]
    pub async fn unsafe_delete(
        &self,
        connection: &mut DatabaseConnection,
    ) -> Result<(), DatabaseError> {
        if matches!(self.state, FileVersionState::Ready) {
            return DatabaseError::PreconditionError(
                "Versions in ready state cannot be deleted".to_string(),
            )
            .err();
        }

        let fv_id = self.id;

        connection
            .transaction(|tx| {
                Box::pin(async move {
                    sqlx::query!(
                        "DELETE FROM file_version_tag WHERE file_version_id = ?1",
                        fv_id,
                    )
                    .execute(&mut **tx)
                    .await?;

                    sqlx::query!("DELETE FROM file_version WHERE id = ?1", fv_id)
                        .execute(&mut **tx)
                        .await?;

                    Ok(())
                })
            })
            .await
    }
}
