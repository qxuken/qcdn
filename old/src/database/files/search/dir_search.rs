use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqliteConnection};
use uuid::Uuid;

use crate::grpc::GetDirResponse;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DirSearch {
    pub id: String,
    pub name: String,
}

impl From<DirSearch> for GetDirResponse {
    fn from(value: DirSearch) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl DirSearch {
    pub async fn get_all(connection: &mut SqliteConnection) -> Result<Vec<Self>> {
        let items = sqlx::query_as("SELECT id, dir FROM dir")
            .fetch_all(connection)
            .await?;

        Ok(items)
    }

    pub async fn find_by_id(connection: &mut SqliteConnection, id: &Uuid) -> Result<Option<Self>> {
        let id = id.to_string();

        let item = sqlx::query_as("SELECT id, dir FROM dir WHERE id = ?")
            .bind(id)
            .fetch_optional(connection)
            .await?;

        Ok(item)
    }
}
