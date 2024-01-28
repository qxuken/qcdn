use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqliteConnection};
use uuid::Uuid;

use crate::database::files::file_type::FileType;
use crate::grpc::{self, GetFileResponse};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FileSearch {
    pub id: String,
    pub dir_id: String,
    pub name: String,
    pub file_type: FileType,
}

impl From<FileSearch> for GetFileResponse {
    fn from(value: FileSearch) -> Self {
        let file_type: grpc::FileType = value.file_type.into();
        Self {
            id: value.id,
            dir_id: value.dir_id,
            name: value.name,
            file_type: file_type.into(),
        }
    }
}

impl FileSearch {
    pub async fn get_all(connection: &mut SqliteConnection) -> Result<Vec<Self>> {
        let items = sqlx::query_as("SELECT id, dir_id, name, file_type FROM file")
            .fetch_all(connection)
            .await?;

        Ok(items)
    }

    pub async fn find_by_dir_id(
        connection: &mut SqliteConnection,
        dir_id: &Uuid,
    ) -> Result<Vec<Self>> {
        let dir_id = dir_id.to_string();

        let items = sqlx::query_as("SELECT id, dir_id, name, file_type FROM file WHERE dir_id = ?")
            .bind(dir_id)
            .fetch_all(connection)
            .await?;

        Ok(items)
    }

    pub async fn find_by_id(connection: &mut SqliteConnection, id: &Uuid) -> Result<Option<Self>> {
        let id = id.to_string();

        let item = sqlx::query_as("SELECT id, dir_id, name, file_type FROM file WHERE id = ?")
            .bind(id)
            .fetch_optional(connection)
            .await?;

        Ok(item)
    }
}
