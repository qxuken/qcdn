use chrono::NaiveDateTime;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError, FileVersionState};

#[derive(Debug, Deserialize, Serialize)]
struct FileVersionWithTagsRaw {
    id: i64,
    file_id: i64,
    size: i64,
    hash: String,
    name: String,
    state: FileVersionState,
    tags: Option<String>,
    created_at: NaiveDateTime,
    deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileVersionWithTags {
    pub id: i64,
    pub file_id: i64,
    pub size: i64,
    pub hash: String,
    pub name: String,
    pub state: FileVersionState,
    pub tags: Vec<String>,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl From<FileVersionWithTagsRaw> for FileVersionWithTags {
    fn from(value: FileVersionWithTagsRaw) -> Self {
        Self {
            id: value.id,
            file_id: value.file_id,
            size: value.size,
            hash: value.hash,
            name: value.name,
            state: value.state,
            tags: value
                .tags
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            created_at: value.created_at,
            deleted_at: value.deleted_at,
        }
    }
}

impl FileVersionWithTags {
    #[instrument(skip(connection))]
    pub async fn find_by_file_id(
        connection: &mut DatabaseConnection,
        file_id: &i64,
    ) -> Result<Vec<Self>, DatabaseError> {
        let items = sqlx::query_as!(
            FileVersionWithTagsRaw,
            r#"
            SELECT
                fv.*,
                group_concat(fvt.name) tags
            FROM
                file_version fv
                LEFT JOIN file_version_tag fvt ON fvt.file_version_id = fv.id
            WHERE fv.file_id = ?
            GROUP BY fv.id"#,
            file_id,
        )
        .fetch_all(connection)
        .await?
        .into_iter()
        .map(FileVersionWithTags::from)
        .collect();

        Ok(items)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_id(
        connection: &mut DatabaseConnection,
        id: &i64,
    ) -> Result<Self, DatabaseError> {
        let item = sqlx::query_as!(
            FileVersionWithTagsRaw,
            r#"
            SELECT
                fv.*,
                group_concat(fvt.name) tags
            FROM
                file_version fv
                LEFT JOIN file_version_tag fvt ON fvt.file_version_id = fv.id
            WHERE fv.id = ?
            GROUP BY fv.id"#,
            id,
        )
        .fetch_one(connection)
        .await?
        .into();

        Ok(item)
    }
}
