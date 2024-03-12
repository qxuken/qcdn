use chrono::NaiveDateTime;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError, FileVersionState};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileVersionMeta {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub media_type: String,
    pub hash: String,
    pub path: String,
}

impl FileVersionMeta {
    #[instrument(skip(connection))]
    pub async fn find_by_id(
        connection: &mut DatabaseConnection,
        id: &i64,
    ) -> Result<Self, DatabaseError> {
        let item = sqlx::query!(
            r#"
              SELECT
                  fv.created_at,
                  fv.hash,
                  f.media_type,
                  d.id dir,
                  f.id file,
                  fv.id version
              FROM
                  file_version fv
                  INNER JOIN file f ON f.id = fv.file_id
                  INNER JOIN dir d ON d.id = f.dir_id
              WHERE
                fv.id = ?
                AND fv.state = ?
                AND fv.deleted_at IS NULL
          "#,
            id,
            FileVersionState::Ready,
        )
        .fetch_one(connection)
        .await?;

        Ok(Self {
            id: item.version,
            created_at: item.created_at,
            media_type: item.media_type,
            hash: item.hash,
            path: format!("{}/{}/{}", item.dir, item.file, item.version),
        })
    }

    #[instrument(skip(connection))]
    pub async fn find_by_path(
        connection: &mut DatabaseConnection,
        dir_name: &str,
        file_name: &str,
        version_or_tag: &str,
    ) -> Result<Self, DatabaseError> {
        let item = sqlx::query!(
            r#"
              SELECT
                  fv.created_at,
                  fv.hash,
                  f.media_type,
                  d.id dir,
                  f.id file,
                  fv.id version
              FROM
                  file_version fv
                  INNER JOIN file f ON f.id = fv.file_id
                  INNER JOIN dir d ON d.id = f.dir_id
                  LEFT JOIN file_version_tag fvt ON fvt.file_version_id = fv.id
              WHERE
                d.name = ?1
                AND f.name = ?2
                AND (fv.name = ?3 OR fvt.name = ?3)
                AND fv.state = ?4
                AND fv.deleted_at IS NULL
          "#,
            dir_name,
            file_name,
            version_or_tag,
            FileVersionState::Ready,
        )
        .fetch_one(connection)
        .await?;

        Ok(Self {
            id: item.version,
            created_at: item.created_at,
            media_type: item.media_type,
            hash: item.hash,
            path: format!("{}/{}/{}", item.dir, item.file, item.version),
        })
    }
}
