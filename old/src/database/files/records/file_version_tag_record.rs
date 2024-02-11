use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqliteConnection};
use uuid::Uuid;

use crate::database::utils;

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[repr(i32)]
pub enum FileVersionState {
    Created,
    Downloading,
    Ready,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileVersionTagRecord {
    pub id: Uuid,
    pub file_version_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub activated_at: DateTime<Utc>,
}

impl FileVersionTagRecord {
    pub async fn find_by_name(
        connection: &mut SqliteConnection,
        file_version_id: &Uuid,
        name: &str,
    ) -> Result<Option<Self>> {
        let file_version_id = file_version_id.to_string();

        let item = sqlx::query_as(
            r#"
            SELECT *
            FROM file_version_tag
            WHERE
                file_version_id = ?
                AND name = ?
            "#,
        )
        .bind(file_version_id)
        .bind(name)
        .fetch_optional(connection)
        .await?;

        Ok(item)
    }

    pub async fn create(
        connection: &mut SqliteConnection,
        file_version_id: &Uuid,
        name: &str,
        ts: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let id = uuid::Uuid::now_v7().to_string();
        let file_version_id = file_version_id.to_string();

        let ts = ts.unwrap_or_else(Utc::now).timestamp();

        let item = sqlx::query_as(
            r#"
            INSERT INTO file_version_tag(id, file_version_id, name, created_at, activated_at)
            VALUES (?1, ?2, ?3, ?4, ?4)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(file_version_id)
        .bind(name)
        .bind(ts)
        .fetch_one(&mut *connection)
        .await?;

        Ok(item)
    }

    pub async fn create_or_move(
        connection: &mut SqliteConnection,
        file_version_id: &Uuid,
        name: &str,
        ts: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let item = match Self::find_by_name(connection, file_version_id, name).await? {
            Some(mut tag) => {
                tag.move_to_version(connection, file_version_id, ts).await?;
                tag
            }
            None => Self::create(connection, file_version_id, name, ts).await?,
        };

        Ok(item)
    }
}

impl FileVersionTagRecord {
    pub async fn move_to_version(
        &mut self,
        connection: &mut SqliteConnection,
        file_version_id: &Uuid,
        ts: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let id = self.id.to_string();
        let new_file_version_id = file_version_id.to_string();

        let ts = ts.unwrap_or_else(Utc::now).timestamp();

        sqlx::query!(
            "UPDATE file_version_tag SET file_version_id = ?2, activated_at = ?3 WHERE id = ?1",
            id,
            new_file_version_id,
            ts,
        )
        .execute(connection)
        .await?;

        self.file_version_id = *file_version_id;

        Ok(())
    }
}

impl FromRow<'_, SqliteRow> for FileVersionTagRecord {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id = utils::parse_uuid(row, "id")?;
        let file_version_id = utils::parse_uuid(row, "file_version_id")?;

        let created_at = utils::parse_timestamp(row, "created_at")?;
        let activated_at = utils::parse_timestamp(row, "activated_at")?;

        Ok(Self {
            id,
            file_version_id,
            name: row.try_get("name")?,
            created_at,
            activated_at,
        })
    }
}
