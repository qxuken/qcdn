use chrono::NaiveDateTime;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use tracing::instrument;

pub use dir_upsert::DirUpsert;

use crate::DatabaseError;

mod dir_upsert;

#[derive(Debug, Deserialize, Serialize)]
pub struct Dir {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl Dir {
    #[instrument(skip(connection))]
    pub async fn get_all(connection: &mut SqliteConnection) -> Result<Vec<Self>, DatabaseError> {
        let items = sqlx::query_as!(Self, "SELECT * FROM dir")
            .fetch_all(connection)
            .await?;

        Ok(items)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_id(
        connection: &mut SqliteConnection,
        id: i64,
    ) -> Result<Option<Self>, DatabaseError> {
        let item = sqlx::query_as!(Self, "SELECT * FROM dir WHERE id = ?", id)
            .fetch_optional(connection)
            .await?;

        Ok(item)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_name(
        connection: &mut SqliteConnection,
        name: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        let item = sqlx::query_as!(Self, "SELECT * FROM dir WHERE name = ?", name)
            .fetch_optional(connection)
            .await?;

        Ok(item)
    }
}

impl Dir {
    #[instrument(skip(connection))]
    pub async fn delete_if_no_files_exists(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            DELETE FROM dir
            WHERE
                id = ?1
                AND (SELECT COUNT(*) FROM file WHERE dir_id = ?1) = 0
            "#,
            self.id
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}
