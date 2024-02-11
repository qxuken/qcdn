use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqliteConnection};
use uuid::Uuid;

use crate::database::utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct DirRecord {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl DirRecord {
    pub async fn find_by_name(
        connection: &mut SqliteConnection,
        name: &str,
    ) -> Result<Option<Self>> {
        let item = sqlx::query_as("SELECT * FROM dir WHERE name = ?")
            .bind(name)
            .fetch_optional(connection)
            .await?;

        Ok(item)
    }

    pub async fn create(
        connection: &mut SqliteConnection,
        name: &str,
        ts: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let uuid = uuid::Uuid::now_v7().to_string();
        let created_at = ts.unwrap_or_else(Utc::now).timestamp();

        let item = sqlx::query_as(
            r#"
            INSERT INTO dir(id, name, created_at)
            VALUES (?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(uuid)
        .bind(name)
        .bind(created_at)
        .fetch_one(connection)
        .await?;

        Ok(item)
    }

    pub async fn find_by_name_or_create(
        connection: &mut SqliteConnection,
        dir: &str,
        ts: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let item = match Self::find_by_name(connection, dir).await? {
            Some(dir) => dir,
            None => Self::create(connection, dir, ts).await?,
        };

        Ok(item)
    }
}

impl DirRecord {
    pub async fn delete_if_no_files_exists(&self, connection: &mut SqliteConnection) -> Result<()> {
        let id = self.id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM dir
            WHERE
                id = ?1
                AND (SELECT COUNT(*) FROM file WHERE dir_id = ?1) = 0
            "#,
            id
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}

impl FromRow<'_, SqliteRow> for DirRecord {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id = utils::parse_uuid(row, "id")?;

        let created_at = utils::parse_timestamp(row, "created_at")?;

        Ok(Self {
            id,
            name: row.try_get("name")?,
            created_at,
        })
    }
}
