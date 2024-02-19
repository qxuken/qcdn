use color_eyre::Result;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::path::PathBuf;
use tracing::instrument;

pub use constants::*;
pub use error::*;
pub use models::*;

pub mod constants;
pub mod error;
pub mod models;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

pub type DatabaseConnection = sqlx::pool::PoolConnection<sqlx::Sqlite>;

#[derive(Debug, Clone)]
pub struct Database(SqlitePool);

impl Database {
    #[instrument]
    pub async fn try_from_path(path: &PathBuf) -> Result<Self, DatabaseError> {
        tracing::trace!("Creating database pool");
        let res = SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
        )
        .await
        .map(Self)?;
        tracing::info!("Created database pool");
        tracing::trace!("{res:#?}");
        Ok(res)
    }
}

impl Database {
    #[instrument(skip(self))]
    pub fn inner(&self) -> SqlitePool {
        self.0.clone()
    }

    #[instrument(skip(self))]
    pub async fn establish_connection(&self) -> Result<DatabaseConnection, DatabaseError> {
        tracing::trace!("Establishing database connection");
        let connection = self.0.acquire().await?;
        Ok(connection)
    }

    pub async fn run_migrations(self) -> Result<Self> {
        tracing::debug!("Running migrations");
        MIGRATOR.run(&self.0).await?;
        tracing::info!("Database migrated");
        Ok(self)
    }
}
