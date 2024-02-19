use chrono::{NaiveDateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use tracing::instrument;

use crate::DatabaseError;

use super::Dir;

#[derive(Debug, Deserialize, Serialize)]
pub struct DirUpsert {
    pub name: String,
    pub created_at: Option<NaiveDateTime>,
}

impl DirUpsert {
    #[instrument(skip(connection))]
    async fn create(self, connection: &mut SqliteConnection) -> Result<Dir, DatabaseError> {
        let created_at = self.created_at.unwrap_or_else(|| Utc::now().naive_utc());

        let item = sqlx::query_as!(
            Dir,
            r#"
                INSERT INTO dir(name, created_at)
                VALUES (?, ?)
                RETURNING *
            "#,
            self.name,
            created_at
        )
        .fetch_one(connection)
        .await?;

        Ok(item)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_name_or_create(
        self,
        connection: &mut SqliteConnection,
    ) -> Result<Dir, DatabaseError> {
        let item = match Dir::find_by_name(connection, &self.name).await? {
            Some(dir) => dir,
            None => self.create(connection).await?,
        };

        Ok(item)
    }
}
