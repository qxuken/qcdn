use chrono::{NaiveDateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError};

use super::{FileVersion, FileVersionState};

#[derive(Debug, Deserialize, Serialize)]
pub struct NewFileVersion {
    pub file_id: i64,
    pub size: i64,
    pub name: String,
    pub state: FileVersionState,
    pub created_at: Option<NaiveDateTime>,
}

impl NewFileVersion {
    #[instrument(skip(connection))]
    pub async fn create(
        self,
        connection: &mut DatabaseConnection,
    ) -> Result<FileVersion, DatabaseError> {
        if FileVersion::find_ready(connection, &self.file_id, &self.name)
            .await?
            .is_some()
        {
            return DatabaseError::PreconditionError("Version is already exists".to_string()).err();
        }

        let created_at = self.created_at.unwrap_or_else(|| Utc::now().naive_utc());

        let item = sqlx::query_as!(
            FileVersion,
            r#"
            INSERT INTO file_version(file_id, size, name, state, created_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
            self.file_id,
            self.size,
            self.name,
            self.state,
            created_at,
        )
        .fetch_one(&mut *connection)
        .await?;

        Ok(item)
    }
}
