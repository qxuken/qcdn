use chrono::{NaiveDateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError};

pub use file_version_tag_upsert::FileVersionTagUpsert;

mod file_version_tag_upsert;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileVersionTag {
    pub id: i64,
    pub file_version_id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub activated_at: NaiveDateTime,
}

impl FileVersionTag {
    #[instrument(skip(connection))]
    pub async fn find_by_file_version(
        connection: &mut DatabaseConnection,
        file_version_id: &i64,
    ) -> Result<Vec<Self>, DatabaseError> {
        let items = sqlx::query_as!(
            Self,
            "SELECT * FROM file_version_tag WHERE file_version_id = ?",
            file_version_id,
        )
        .fetch_all(connection)
        .await?;

        Ok(items)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_file_and_name(
        connection: &mut DatabaseConnection,
        file_id: &i64,
        name: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        let items = sqlx::query_as!(
            Self,
            r#"SELECT fvt.*
            FROM
                file_version_tag fvt
                INNER JOIN file_version fv ON fv.id = fvt.file_version_id
            WHERE fv.file_id = ? AND fvt.name = ?"#,
            file_id,
            name
        )
        .fetch_optional(connection)
        .await?;

        Ok(items)
    }
}

impl FileVersionTag {
    #[instrument(skip(connection))]
    pub async fn move_to_version(
        &mut self,
        connection: &mut DatabaseConnection,
        file_version_id: &i64,
    ) -> Result<(), DatabaseError> {
        let now = Utc::now().naive_utc();

        sqlx::query!(
            "UPDATE file_version_tag SET file_version_id = ?2, activated_at = ?3  WHERE id = ?1",
            self.id,
            file_version_id,
            now,
        )
        .execute(connection)
        .await?;

        self.activated_at = now;
        self.file_version_id = *file_version_id;
        Ok(())
    }
}
