use anyhow::Result;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqliteConnection};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dir {
    pub path: String,
    pub created_at: DateTime<Utc>,
}

impl Dir {
    pub async fn find(connection: &mut SqliteConnection, path: &str) -> Result<Option<Dir>> {
        let item = sqlx::query_as("SELECT * FROM dir WHERE path = ?1")
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
        let dir = match Dir::find(connection, &path).await? {
            Some(dir) => dir,
            None => Dir::create(connection, path).await?,
        };

        Ok(dir)
    }

    pub async fn delete(connection: &mut SqliteConnection, path: &str) -> Result<()> {
        sqlx::query!("DELETE FROM dir WHERE path = ?1", path)
            .execute(connection)
            .await?;
        Ok(())
    }
}

impl Dir {
    pub async fn del(&self, connection: &mut SqliteConnection) -> Result<()> {
        sqlx::query!("DELETE FROM dir WHERE path = ?1", self.path)
            .execute(connection)
            .await?;

        Ok(())
    }
}

impl FromRow<'_, SqliteRow> for Dir {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
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
            path: row.try_get("path")?,
            created_at,
        })
    }
}
