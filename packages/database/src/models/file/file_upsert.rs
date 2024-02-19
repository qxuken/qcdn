use chrono::{NaiveDateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use tracing::instrument;

use crate::DatabaseError;

use super::{File, FileType};

#[derive(Debug, Deserialize, Serialize)]
pub struct FileUpsert {
    pub dir_id: i64,
    pub name: String,
    pub file_type: FileType,
    pub created_at: Option<NaiveDateTime>,
}

impl FileUpsert {
    #[instrument(skip(connection))]
    async fn create(self, connection: &mut SqliteConnection) -> Result<File, DatabaseError> {
        let created_at = self.created_at.unwrap_or_else(|| Utc::now().naive_utc());

        let item = sqlx::query_as!(
            File,
            r#"
                INSERT INTO file(dir_id, name, file_type, created_at)
                VALUES (?, ?, ?, ?)
                RETURNING *
            "#,
            self.dir_id,
            self.name,
            self.file_type,
            created_at,
        )
        .fetch_one(connection)
        .await?;

        Ok(item)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_name_or_create(
        self,
        connection: &mut SqliteConnection,
    ) -> Result<File, DatabaseError> {
        let item = match File::find_by_dir_and_name(connection, &self.dir_id, &self.name).await? {
            Some(id) => id,
            None => self.create(connection).await?,
        };

        Ok(item)
    }
}
