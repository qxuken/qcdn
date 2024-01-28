use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqliteConnection};

use crate::grpc::{self, GetFileVersionsResponseItem, GetFilesResponseItem};

use super::file_type::FileType;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FileSearch {
    pub id: String,
    pub dir: String,
    pub name: String,
    pub file_type: FileType,
}

impl From<FileSearch> for GetFilesResponseItem {
    fn from(value: FileSearch) -> Self {
        let file_type: grpc::FileType = value.file_type.into();
        Self {
            id: value.id,
            dir: value.dir,
            name: value.name,
            file_type: file_type.into(),
        }
    }
}

impl FileSearch {
    pub async fn get_all(connection: &mut SqliteConnection) -> Result<Vec<Self>> {
        let items = sqlx::query_as("SELECT id, dir, name, file_type FROM file")
            .fetch_all(connection)
            .await?;

        Ok(items)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FileVersionSearch {
    pub id: String,
    pub version: String,
    pub is_latest: bool,
    pub is_deleted: bool,
}

impl From<FileVersionSearch> for GetFileVersionsResponseItem {
    fn from(value: FileVersionSearch) -> Self {
        Self {
            id: value.id,
            version: value.version,
            is_latest: value.is_latest,
            is_deleted: value.is_deleted,
        }
    }
}

impl FileVersionSearch {
    pub async fn find_by_file_id(
        connection: &mut SqliteConnection,
        file_id: &str,
    ) -> Result<Vec<Self>> {
        let items = sqlx::query_as(
            r#"SELECT
	fv.id,
	fv.version,
	fv.deleted_at IS NOT NULL is_deleted,
	flv.id IS NOT NULL AND flv.expired_at IS NULL is_latest
FROM
	file_version fv
	LEFT JOIN file_latest_version flv ON flv.file_version_id = fv.id
WHERE fv.file_id = ?1"#,
        )
        .bind(file_id)
        .fetch_all(connection)
        .await?;

        Ok(items)
    }
}
