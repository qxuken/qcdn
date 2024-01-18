use anyhow::Result;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqliteConnection};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dir {
    pub id: Uuid,
    pub path: String,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, SqliteRow> for Dir {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id: String = row.try_get("id")?;
        let id: Uuid = uuid::Uuid::parse_str(&id).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: e.into(),
        })?;

        let created_at: i64 = row.try_get("created_at")?;
        let created_at: DateTime<Utc> = match Utc.timestamp_opt(created_at, 0) {
            LocalResult::Single(res) => Ok(res),
            LocalResult::Ambiguous(min, _max) => Ok(min),
            LocalResult::None => {
                let err = anyhow::anyhow!("created_at decode error");
                Err(sqlx::Error::ColumnDecode {
                    index: "created_at".to_string(),
                    source: err.into(),
                })
            }
        }?;

        Ok(Self {
            id,
            path: row.try_get("path")?,
            created_at,
        })
    }
}

impl Dir {
    pub async fn find(connection: &mut SqliteConnection, path: &str) -> Result<Option<Dir>> {
        let item: Option<Dir> = sqlx::query_as("SELECT * FROM dir WHERE path = ?1")
            .bind(path)
            .fetch_optional(&mut *connection)
            .await?;

        Ok(item)
    }

    async fn create(connection: &mut SqliteConnection, path: &str) -> Result<Dir> {
        let uuid = uuid::Uuid::now_v7().to_string();
        let created_at = Utc::now().timestamp();

        let item_id: Dir =
            sqlx::query_as("INSERT INTO dir(id, path, created_at) VALUES (?1, ?2, ?3) RETURNING *")
                .bind(uuid)
                .bind(path)
                .bind(created_at)
                .fetch_one(&mut *connection)
                .await?;
        Ok(item_id)
    }

    pub async fn find_or_create(connection: &mut SqliteConnection, path: &str) -> Result<Dir> {
        let dir = match Dir::find(connection, path).await? {
            Some(dir) => dir,
            None => Dir::create(connection, path).await?,
        };

        Ok(dir)
    }

    pub async fn delete_dir(connection: &mut SqliteConnection, path: &str) -> Result<()> {
        sqlx::query!("DELETE FROM dir WHERE path = ?1", path)
            .execute(connection)
            .await?;

        Ok(())
    }
}

impl Dir {
    pub async fn delete(&self, connection: &mut SqliteConnection) -> Result<()> {
        let id = self.id.to_string();
        sqlx::query!("DELETE FROM dir WHERE id = ?1", id)
            .execute(connection)
            .await?;

        Ok(())
    }
}
