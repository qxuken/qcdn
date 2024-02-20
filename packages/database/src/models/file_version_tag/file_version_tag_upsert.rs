use chrono::{NaiveDateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError};

use super::FileVersionTag;

#[derive(Debug, Deserialize, Serialize)]
pub struct FileVersionTagUpsert {
    pub file_version_id: i64,
    pub name: String,
    pub created_at: Option<NaiveDateTime>,
}

impl FileVersionTagUpsert {
    #[instrument(skip(connection))]
    async fn create(
        self,
        connection: &mut DatabaseConnection,
    ) -> Result<FileVersionTag, DatabaseError> {
        let created_at = self.created_at.unwrap_or_else(|| Utc::now().naive_utc());

        let item = sqlx::query_as!(
            FileVersionTag,
            r#"
            INSERT INTO file_version_tag(file_version_id, name, created_at, activated_at)
            VALUES (?1, ?2, ?3, ?3)
            RETURNING *
            "#,
            self.file_version_id,
            self.name,
            created_at,
        )
        .fetch_one(&mut *connection)
        .await?;

        Ok(item)
    }

    #[instrument(skip(connection))]
    pub async fn create_or_move(
        self,
        connection: &mut DatabaseConnection,
    ) -> Result<FileVersionTag, DatabaseError> {
        let item = match FileVersionTag::find_by_version_and_name(
            connection,
            &self.file_version_id,
            &self.name,
        )
        .await?
        {
            Some(mut tag) => {
                tag.move_to_version(connection, &self.file_version_id)
                    .await?;
                tag
            }
            None => self.create(connection).await?,
        };

        Ok(item)
    }
}
