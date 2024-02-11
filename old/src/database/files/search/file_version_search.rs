use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row, SqliteConnection};
use uuid::Uuid;

use crate::grpc::GetFileVersionResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileVersionSearch {
    pub id: String,
    pub file_id: String,
    pub version: String,
    pub size: u64,
    pub tags: Vec<String>,
    pub is_deleted: bool,
}

impl From<FileVersionSearch> for GetFileVersionResponse {
    fn from(value: FileVersionSearch) -> Self {
        Self {
            id: value.id,
            file_id: value.file_id,
            version: value.version,
            size: value.size,
            tags: value.tags,
            is_deleted: value.is_deleted,
        }
    }
}

impl FileVersionSearch {
    pub async fn find_by_file_id(
        connection: &mut SqliteConnection,
        file_id: &Uuid,
    ) -> Result<Vec<Self>> {
        let file_id = file_id.to_string();
        let items = sqlx::query_as(
            r#"
            SELECT
                fv.id,
                fv.file_id,
                fv.version,
                fv.size,
                GROUP_CONCAT(fvt.name) tags,
                fv.deleted_at IS NOT NULL is_deleted
            FROM
                file_version fv
                LEFT JOIN file_version_tag fvt ON fvt.file_version_id = fv.id
            WHERE fv.file_id = ?
            GROUP BY fv.id"#,
        )
        .bind(file_id)
        .fetch_all(connection)
        .await?;

        Ok(items)
    }

    pub async fn find_by_id(connection: &mut SqliteConnection, id: &Uuid) -> Result<Option<Self>> {
        let id = id.to_string();

        let items = sqlx::query_as(
            r#"
            SELECT
                fv.id,
                fv.file_id,
                fv.version,
                fv.size,
                GROUP_CONCAT(fvt.name) tags,
                fv.deleted_at IS NOT NULL is_deleted
            FROM
                file_version fv
                LEFT JOIN file_version_tag fvt ON fvt.file_version_id = fv.id
            WHERE fv.id = ?
            GROUP BY fv.id"#,
        )
        .bind(id)
        .fetch_optional(connection)
        .await?;

        Ok(items)
    }
}

impl FromRow<'_, SqliteRow> for FileVersionSearch {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let size: i64 = row.try_get("size")?;
        let size = size as u64;

        Ok(Self {
            id: row.try_get("id")?,
            file_id: row.try_get("file_id")?,
            version: row.try_get("version")?,
            size,
            is_deleted: row.try_get("is_deleted")?,
            tags: row
                .try_get("tags")
                .map(|s: &str| s.split(',').map(String::from).collect())?,
        })
    }
}
