use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{Connection, FromRow, Row, SqliteConnection};
use uuid::Uuid;

use crate::database::utils;

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[repr(i32)]
pub enum FileVersionState {
    Ready,
    Downloading,
    Created,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileVersionRecord {
    pub id: Uuid,
    pub file_id: Uuid,
    pub size: u64,
    pub version: String,
    pub state: FileVersionState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: DateTime<Utc>,
}

impl FileVersionRecord {
    pub async fn find_by_id(connection: &mut SqliteConnection, id: Uuid) -> Result<Option<Self>> {
        let id = id.to_string();
        let item = sqlx::query_as("SELECT * FROM file_version WHERE id = ?1")
            .bind(id)
            .fetch_optional(&mut *connection)
            .await?;
        Ok(item)
    }

    pub async fn find_by_path_exact(
        connection: &mut SqliteConnection,
        dir: &str,
        name: &str,
        version: &str,
    ) -> Result<Option<Self>> {
        let item = sqlx::query_as(
            r#"SELECT file_version.*
FROM
    file
    LEFT JOIN file_version ON
    file_version.file_id = file.id
    AND file_latest_version.deleted_at IS NULL
WHERE
    file.dir = ?1
    AND file.name = ?2
    AND file_version.version = ?3
    AND file_version.state = ?4"#,
        )
        .bind(dir)
        .bind(name)
        .bind(version)
        .bind(FileVersionState::Ready)
        .fetch_optional(connection)
        .await?;

        Ok(item)
    }

    pub async fn find_by_path_latest(
        connection: &mut SqliteConnection,
        dir: &str,
        name: &str,
    ) -> Result<Option<Self>> {
        let item = sqlx::query_as(
            r#"SELECT file_version.*
FROM
    file
    LEFT JOIN file_version ON file_version.file_id = file.id AND file_latest_version.deleted_at IS NULL
    LEFT JOIN file_latest_version ON
        file_latest_version.file_id = file.id
        AND file_latest_version.file_version_id = file_version.id
        AND file_latest_version.deleted_at IS NULL
WHERE
    file.dir = ?1
    AND file.name = ?2
    AND file_version.state = ?3"#,
        )
        .bind(dir)
        .bind(name)
        .bind(FileVersionState::Ready)
        .fetch_optional(connection)
        .await?;

        Ok(item)
    }

    pub async fn find_by_path(
        connection: &mut SqliteConnection,
        dir: &str,
        name: &str,
        version: Option<&str>,
    ) -> Result<Option<Self>> {
        let item = match version {
            Some(version) => Self::find_by_path_exact(connection, dir, name, version).await?,
            None => Self::find_by_path_latest(connection, dir, name).await?,
        };
        Ok(item)
    }

    pub async fn create(
        connection: &mut SqliteConnection,
        file_id: &Uuid,
        version: &str,
        size: u64,
        state: FileVersionState,
    ) -> Result<Self> {
        let file_id = file_id.to_string();

        if sqlx::query!(
            r#"SELECT true
FROM file_version fv
WHERE file_id = ?1 AND version = ?2 AND deleted_at IS NULL"#,
            file_id,
            version
        )
        .fetch_optional(&mut *connection)
        .await?
        .is_some()
        {
            bail!("Version already exists")
        }

        let id = uuid::Uuid::now_v7().to_string();
        let created_at = Utc::now().timestamp();

        let item = sqlx::query_as(
            r#"INSERT INTO
file_version(id, file_id, size, version, state, created_at, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
RETURNING *"#,
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
    pub async fn update_state(
        &mut self,
        connection: &mut SqliteConnection,
        state: FileVersionState,
    ) -> Result<()> {
        let file_version_id = self.id.to_string();
        let updated_at = Utc::now().timestamp();

        sqlx::query!(
            "UPDATE file_version SET state = ?2, updated_at = ?3 WHERE id = ?1",
            file_version_id,
            state,
            updated_at
        )
        .execute(connection)
        .await?;

        self.state = state;

        Ok(())
    }

    pub async fn delete(&self, connection: &mut SqliteConnection) -> Result<()> {
        let file_version_id = self.id.to_string();
        let file_id = self.file_id.to_string();
        let deleted_at = Utc::now().timestamp();
        let is_latest = self.is_latest(connection).await?;

        connection
            .transaction(|tx| {
                Box::pin(async move {
                    sqlx::query!(
                        "UPDATE file_version SET deleted_at = ?2 WHERE id = ?1",
                        file_version_id,
                        deleted_at
                    )
                    .execute(&mut **tx)
                    .await?;

                    if !is_latest {
                        return Ok(());
                    }

                    if let Some(prev_latest_id) = sqlx::query!(
                        r#"SELECT id
FROM file_latest_version
WHERE file_id = ?1 AND expired_at IS NOT NULL ORDER BY expired_at DESC"#,
                        file_id,
                    )
                    .fetch_optional(&mut **tx)
                    .await?
                    {
                        sqlx::query!(
                            "UPDATE file_latest_version SET expired_at = NULL WHERE id = ?1",
                            prev_latest_id.id,
                        )
                        .execute(&mut **tx)
                        .await?;
                    }

                    sqlx::query!(
                        "UPDATE file_latest_version SET expired_at = ?2 WHERE file_version_id = ?1",
                        file_version_id,
                        deleted_at,
                    )
                    .execute(&mut **tx)
                    .await?;
                    Ok(())
                })
            })
            .await
    }

    pub async fn force_delete(&self, connection: &mut SqliteConnection) -> Result<()> {
        let file_version_id = self.id.to_string();
        sqlx::query!("DELETE FROM file_version WHERE id = ?1", file_version_id,)
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn is_latest(&self, connection: &mut SqliteConnection) -> Result<bool> {
        let file_version_id = self.id.to_string();
        let file_id = self.file_id.to_string();

        let res = sqlx::query!(
            r#"SELECT true
FROM file_latest_version
WHERE
    file_id = ?1
    AND file_version_id = ?2
    AND expired_at IS NULL"#,
            file_id,
            file_version_id,
        )
        .fetch_optional(connection)
        .await?;

        Ok(res.is_some())
    }

    pub async fn make_latest(&mut self, connection: &mut SqliteConnection) -> Result<()> {
        let file_version_id = self.id.to_string();
        let file_id = self.file_id.to_string();
        let expired_at = Utc::now().timestamp();

        connection
            .transaction(|tx| {
                Box::pin(async move {
                    sqlx::query!(
                        r#"UPDATE file_latest_version SET expired_at = ?2
    WHERE file_id = ?1 AND expired_at IS NULL"#,
                        file_id,
                        expired_at,
                    )
                    .execute(&mut **tx)
                    .await?;

                    let file_latest_version_id = uuid::Uuid::now_v7().to_string();
                    sqlx::query!(
                        r#"INSERT INTO file_latest_version(id, file_id, file_version_id, created_at)
    VALUES(?, ?, ?, ?)"#,
                        file_latest_version_id,
                        file_id,
                        file_version_id,
                        expired_at,
                    )
                    .execute(&mut **tx)
                    .await?;

                    Ok(())
                })
            })
            .await
    }
}

impl FromRow<'_, SqliteRow> for FileVersionRecord {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id = utils::parse_uuid(row, "id")?;
        let file_id = utils::parse_uuid(row, "file_id")?;

        let size: i64 = row.try_get("size")?;
        let size = size as u64;

        let created_at = utils::parse_timestamp(row, "created_at")?;
        let updated_at = utils::parse_timestamp(row, "updated_at")?;
        let deleted_at = utils::parse_timestamp(row, "deleted_at")?;

        Ok(Self {
            id,
            file_id,
            size,
            version: row.try_get("version")?,
            state: row.try_get("state")?,
            created_at,
            updated_at,
            deleted_at,
        })
    }
}
