use anyhow::Result;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqliteConnection};
use uuid::Uuid;

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
pub enum FileType {
    Files,
    Stylesheets,
    Javascript,
    Image,
    Font,
}

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
pub enum FileState {
    Ready,
    Uploading,
    Downloading,
    Created,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: Uuid,
    pub directory_path: String,
    pub name: String,
    pub file_type: FileType,
    pub version: String,
    pub size: u64,
    pub state: FileState,
    pub meta: Value,
    pub created_at: DateTime<Utc>,
}

pub struct CreateFile {
    pub directory_path: String,
    pub name: String,
    pub file_type: FileType,
    pub version: String,
    pub size: u64,
    pub state: FileState,
    pub meta: Value,
}

pub struct FilePath {
    pub directory_path: String,
    pub name: String,
    pub file_type: FileType,
    pub version: Option<String>,
}

impl File {
    pub async fn find_by_id(connection: &mut SqliteConnection, id: Uuid) -> Result<Self> {
        let id: String = id.to_string();
        let item = sqlx::query_as(
            r#"SELECT *
            FROM file
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_one(&mut *connection)
        .await?;
        Ok(item)
    }

    pub async fn find_by_path(connection: &mut SqliteConnection, path: FilePath) -> Result<Self> {
        let item = sqlx::query_as(
            r#"SELECT *
            FROM file
            WHERE 
                directory_path = ?1
                AND name = ?2
                AND file_type = ?3
                AND version = ?4
            "#,
        )
        .bind(path.directory_path)
        .bind(path.name)
        .bind(path.file_type)
        .bind(
            path.version
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("latest"),
        )
        .fetch_one(&mut *connection)
        .await?;

        Ok(item)
    }

    pub async fn create(connection: &mut SqliteConnection, file: CreateFile) -> Result<Self> {
        let uuid = uuid::Uuid::now_v7().to_string();
        let created_at = Utc::now().timestamp();

        let item = sqlx::query_as(
            r#"INSERT INTO
            file(id, directory_path, name, file_type, version, size, state, meta, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            RETURNING *
            "#,
        )
        .bind(uuid)
        .bind(&file.directory_path)
        .bind(&file.name)
        .bind(&file.file_type)
        .bind(&file.version)
        .bind(file.size as i64)
        .bind(&file.state)
        .bind(&file.meta)
        .bind(created_at)
        .fetch_one(&mut *connection)
        .await?;
        Ok(item)
    }
}

impl FromRow<'_, SqliteRow> for File {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id: String = row.try_get("id")?;
        let id: Uuid = uuid::Uuid::parse_str(&id).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: e.into(),
        })?;

        let size: i64 = row.try_get("size")?;
        let size = size as u64;

        let created_at: i64 = row.try_get("created_at")?;
        let created_at: DateTime<Utc> = match Utc.timestamp_opt(created_at, 0) {
            LocalResult::Single(res) => Ok(res),
            LocalResult::Ambiguous(min, _max) => Ok(min),
            LocalResult::None => {
                let err = anyhow::anyhow!("created_at decode error");
                Err(sqlx::Error::ColumnDecode {
                    index: "created_at".to_string(),
                    source: err.into(),
                })
            }
        }?;

        Ok(Self {
            id,
            directory_path: row.try_get("directory_path")?,
            name: row.try_get("name")?,
            file_type: row.try_get("file_type")?,
            version: row.try_get("version")?,
            size,
            state: row.try_get("state")?,
            meta: row.try_get("meta")?,
            created_at,
        })
    }
}
