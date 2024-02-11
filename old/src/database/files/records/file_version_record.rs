use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{Connection, FromRow, Row, SqliteConnection};
use uuid::Uuid;

use crate::database::utils;

#[derive(Debug, sqlx::Type, Serialize, Deserialize, PartialEq, Eq)]
#[repr(i32)]
pub enum FileVersionState {
    Created,
    Downloading,
    Ready,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileVersionRecord {
    pub id: Uuid,
    pub file_id: Uuid,
    pub size: u64,
    pub version: String,
    pub state: FileVersionState,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl FileVersionRecord {
    pub async fn find_by_id(connection: &mut SqliteConnection, id: &Uuid) -> Result<Option<Self>> {
        let id = id.to_string();

        let item = sqlx::query_as(
            r#"
            SELECT *
            FROM file_version
            WHERE id = ? AND state = ?
            "#,
        )
        .bind(id)
        .bind(FileVersionState::Ready)
        .fetch_optional(connection)
        .await?;

        Ok(item)
    }

    pub async fn find_by_version(
        connection: &mut SqliteConnection,
        file_id: &Uuid,
        version: &str,
    ) -> Result<Option<Self>> {
        let file_id = file_id.to_string();

        let item = sqlx::query_as(
            r#"
            SELECT *
            FROM file_version
            WHERE
                file_id = ?
                AND version = ?
                AND state = ?
            "#,
        )
        .bind(file_id)
        .bind(version)
        .bind(FileVersionState::Ready)
        .fetch_optional(connection)
        .await?;

        Ok(item)
    }

    pub async fn create(
        connection: &mut SqliteConnection,
        file_id: &Uuid,
        version: &str,
        size: u64,
        state: FileVersionState,
        ts: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let id = uuid::Uuid::now_v7().to_string();
        let file_id = file_id.to_string();

        let created_at = ts.unwrap_or_else(Utc::now).timestamp();

        if sqlx::query!(
            "SELECT true FROM file_version WHERE file_id = ? AND version = ? AND deleted_at IS NULL",
            file_id,
            version,
        )
            .fetch_optional(&mut *connection)
            .await?
            .is_some() {
            bail!("Version is already exists")
        }

        let item = sqlx::query_as(
            r#"
            INSERT INTO file_version(id, file_id, size, version, state, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(file_id)
        .bind(size as i64)
        .bind(version)
        .bind(state)
        .bind(created_at)
        .fetch_one(&mut *connection)
        .await?;

        Ok(item)
    }
}

impl FileVersionRecord {
    pub async fn path(&self, connection: &mut SqliteConnection) -> Result<(String, String)> {
        let file_version_id = self.id.to_string();

        let r = sqlx::query!(
            r#"
                SELECT f.dir_id, fv.id file_version_id
                FROM
                    file_version fv
                    INNER JOIN file f ON f.id = fv.file_id
                WHERE fv.id = ?1"#,
            file_version_id,
        )
        .fetch_one(connection)
        .await?;

        Ok((r.dir_id, r.file_version_id))
    }

    pub async fn update_state(
        &mut self,
        connection: &mut SqliteConnection,
        state: FileVersionState,
    ) -> Result<()> {
        let file_version_id = self.id.to_string();

        sqlx::query!(
            "UPDATE file_version SET state = ?2 WHERE id = ?1",
            file_version_id,
            state,
        )
        .execute(connection)
        .await?;

        self.state = state;

        Ok(())
    }

    pub async fn delete(
        &mut self,
        connection: &mut SqliteConnection,
        ts: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let file_version_id = self.id.to_string();
        let deleted_at = ts.unwrap_or_else(Utc::now);
        let deleted_at_ts = deleted_at.timestamp();

        sqlx::query!(
            "UPDATE file_version SET deleted_at = ?2 WHERE id = ?1",
            file_version_id,
            deleted_at_ts,
        )
        .execute(connection)
        .await?;

        self.deleted_at = Some(deleted_at);

        Ok(())
    }

    pub async fn unsafe_delete(&self, connection: &mut SqliteConnection) -> Result<()> {
        if self.state == FileVersionState::Ready {
            bail!("Versions with ready state cannot be deleted")
        }

        let file_version_id = self.id.to_string();

        connection
            .transaction(|tx| {
                Box::pin(async move {
                    sqlx::query!(
                        "DELETE FROM file_version_tag WHERE file_version_id = ?1",
                        file_version_id,
                    )
                    .execute(&mut **tx)
                    .await?;

                    sqlx::query!("DELETE FROM file_version WHERE id = ?1", file_version_id)
                        .execute(&mut **tx)
                        .await?;

                    anyhow::Ok(())
                })
            })
            .await?;

        Ok(())
    }
}

impl FromRow<'_, SqliteRow> for FileVersionRecord {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id = utils::parse_uuid(row, "id")?;
        let file_id = utils::parse_uuid(row, "file_id")?;

        let size: i64 = row.try_get("size")?;
        let size = size as u64;

        let created_at = utils::parse_timestamp(row, "created_at")?;
        let deleted_at = utils::parse_timestamp(row, "deleted_at").ok();

        Ok(Self {
            id,
            file_id,
            size,
            version: row.try_get("version")?,
            state: row.try_get("state")?,
            created_at,
            deleted_at,
        })
    }
}
