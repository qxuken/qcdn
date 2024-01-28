use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqliteConnection};
use uuid::Uuid;

use crate::database::files::file_type::FileType;
use crate::database::utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: Uuid,
    pub dir_id: Uuid,
    pub name: String,
    pub file_type: FileType,
    pub created_at: DateTime<Utc>,
}

impl FileRecord {
    pub async fn find_by_name(
        connection: &mut SqliteConnection,
        dir_id: &Uuid,
        name: &str,
    ) -> Result<Option<Self>> {
        let dir_id = dir_id.to_string();

        let item = sqlx::query_as("SELECT * FROM file WHERE dir_id = ? AND name = ?")
            .bind(dir_id)
            .bind(name)
            .fetch_optional(connection)
            .await?;

        Ok(item)
    }

    pub async fn create(
        connection: &mut SqliteConnection,
        dir_id: &Uuid,
        name: &str,
        file_type: FileType,
        ts: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let id = uuid::Uuid::now_v7().to_string();
        let dir_id = dir_id.to_string();

        let created_at = ts.unwrap_or_else(Utc::now).timestamp();

        let item = sqlx::query_as(
            r#"
            INSERT INTO file(id, dir_id, name, file_type, created_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(dir_id)
        .bind(name)
        .bind(file_type)
        .bind(created_at)
        .fetch_one(connection)
        .await?;

        Ok(item)
    }

    pub async fn find_by_name_or_create(
        connection: &mut SqliteConnection,
        dir_id: &Uuid,
        name: &str,
        file_type: FileType,
        ts: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let item = match Self::find_by_name(connection, dir_id, name).await? {
            Some(id) => id,
            None => Self::create(connection, dir_id, name, file_type, ts).await?,
        };

        Ok(item)
    }
}

impl FileRecord {
    pub async fn delete_if_no_versions_exists(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<()> {
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

impl FromRow<'_, SqliteRow> for FileRecord {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id = utils::parse_uuid(row, "id")?;
        let dir_id = utils::parse_uuid(row, "dir_id")?;

        let created_at = utils::parse_timestamp(row, "created_at")?;

        Ok(Self {
            id,
            dir_id,
            name: row.try_get("name")?,
            file_type: row.try_get("file_type")?,
            created_at,
        })
    }
}
