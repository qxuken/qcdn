use chrono::NaiveDateTime;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::instrument;

pub use file_type::FileType;
pub use file_upsert::FileUpsert;

use crate::{DatabaseConnection, DatabaseError};

mod file_type;
mod file_upsert;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct File {
    pub id: i64,
    pub dir_id: i64,
    pub name: String,
    pub file_type: FileType,
    pub created_at: NaiveDateTime,
}

impl File {
    #[instrument(skip(connection))]
    pub async fn find_all_by_dir(
        connection: &mut DatabaseConnection,
        dir_id: &i64,
    ) -> Result<Vec<Self>, DatabaseError> {
        let items = sqlx::query_as!(Self, "SELECT * FROM file WHERE dir_id = ?", dir_id)
            .fetch_all(connection)
            .await?;

        Ok(items)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_id(
        connection: &mut DatabaseConnection,
        id: &i64,
    ) -> Result<Self, DatabaseError> {
        let item = sqlx::query_as!(Self, "SELECT * FROM file WHERE id = ?", id)
            .fetch_one(connection)
            .await?;

        Ok(item)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_dir_and_name_optional(
        connection: &mut DatabaseConnection,
        dir_id: &i64,
        name: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        let item = sqlx::query_as!(
            Self,
            "SELECT * FROM file WHERE dir_id = ? AND name = ?",
            dir_id,
            name
        )
        .fetch_optional(connection)
        .await?;

        Ok(item)
    }
}

impl File {
    #[instrument(skip(connection))]
    pub async fn delete_if_no_versions_exists(
        &self,
        connection: &mut DatabaseConnection,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            DELETE FROM file
            WHERE
                id = ?1
                AND (SELECT COUNT(*) FROM file_version WHERE file_id = ?1) = 0
            "#,
        )
        .bind(self.id.to_string())
        .execute(connection)
        .await?;

        Ok(())
    }
}
