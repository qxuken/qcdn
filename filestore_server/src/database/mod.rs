use std::path::PathBuf;

use anyhow::Result;
use sqlx::{migrate::Migrator, sqlite::SqliteConnectOptions, SqlitePool};

pub use connection::{DatabaseConnection, DatabasePoolConnection};

mod connection;

static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Debug, Clone)]
pub struct Database(pub SqlitePool);

impl Database {
    async fn migrate(self) -> Result<Self> {
        tracing::trace!("Migrating database");
        MIGRATOR.run(&self.0).await?;
        tracing::info!("Migrated database");
        Ok(self)
    }
}

impl Database {
    pub async fn create(path: &PathBuf) -> Result<Self> {
        tracing::trace!("Creating database pool at {:?}", path);
        let res = SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
        )
        .await
        .map_err(anyhow::Error::from)
        .map(Self);
        tracing::info!("Created database pool at {:?}", path);
        res
    }

    pub async fn create_and_migrate(path: &PathBuf) -> Result<Self> {
        Self::create(path).await?.migrate().await
    }
}

impl Database {
    pub fn inner(&self) -> SqlitePool {
        self.0.clone()
    }
}
