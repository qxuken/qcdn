use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqliteConnection};
use uuid::Uuid;

use crate::database::utils;

use super::file_type::FileType;
use super::file_version_record::FileVersionRecord;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: Uuid,
    pub dir: String,
    pub name: String,
    pub file_type: FileType,
    pub created_at: DateTime<Utc>,
}

impl FileRecord {
    pub async fn get_all(connection: &mut SqliteConnection) -> Result<Vec<Self>> {
        let items = sqlx::query_as("SELECT * FROM file")
            .fetch_all(&mut *connection)
            .await?;
        Ok(items)
    }

    pub async fn find_by_id(connection: &mut SqliteConnection, id: Uuid) -> Result<Option<Self>> {
        let id = id.to_string();
        let item = sqlx::query_as("SELECT * FROM file WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *connection)
            .await?;
        Ok(item)
    }

    pub async fn find_by_path(
        connection: &mut SqliteConnection,
        dir: &str,
        name: &str,
    ) -> Result<Option<Self>> {
        let item = sqlx::query_as("SELECT * FROM file WHERE dir = ?1 AND name = ?")
            .bind(dir)
            .bind(name)
            .fetch_optional(&mut *connection)
            .await?;
        Ok(item)
    }

    pub async fn create(
        connection: &mut SqliteConnection,
        dir: &str,
        name: &str,
        file_type: FileType,
    ) -> Result<Self> {
        let uuid = uuid::Uuid::now_v7().to_string();
        let created_at = Utc::now().timestamp();

        let item = sqlx::query_as(
            r#"INSERT INTO
file(id, dir, name, file_type, created_at)
VALUES (?, ?, ?, ?, ?)
RETURNING *"#,
        )
        .bind(uuid)
        .bind(dir)
        .bind(name)
        .bind(file_type)
        .bind(created_at)
        .fetch_one(&mut *connection)
        .await?;
        Ok(item)
    }

    pub async fn find_or_create(
        connection: &mut SqliteConnection,
        dir: &str,
        name: &str,
        file_type: FileType,
    ) -> Result<Self> {
        let item = match Self::find_by_path(connection, dir, name).await? {
            Some(id) => id,
            None => Self::create(connection, dir, name, file_type).await?,
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
            r#"DELETE FROM file
WHERE
    id = ?1
    AND (SELECT COUNT(*) FROM file_version WHERE file_id = ?1) = 0"#,
        )
        .bind(self.id.to_string())
        .execute(connection)
        .await?;

        Ok(())
    }

    pub async fn find_all_versions(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<Vec<FileVersionRecord>> {
        let items = sqlx::query_as("SELECT * FROM file_version WHERE file_id = ?")
            .bind(self.id.to_string())
            .fetch_all(connection)
            .await?;

        Ok(items)
    }
}

impl FromRow<'_, SqliteRow> for FileRecord {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id = utils::parse_uuid(row, "id")?;

        let created_at = utils::parse_timestamp(row, "created_at")?;

        Ok(Self {
            id,
            dir: row.try_get("dir")?,
            name: row.try_get("name")?,
            file_type: row.try_get("file_type")?,
            created_at,
        })
    }
}
