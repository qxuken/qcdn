use color_eyre::Result;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{SqliteConnection, SqlitePool};
use std::{fmt::Debug, path::Path};
use tracing::instrument;

pub use constants::*;
pub use error::*;
pub use models::*;

pub mod constants;
pub mod error;
pub mod models;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

pub type DatabaseConnection = SqliteConnection;
pub type DatabasePoolConnection = sqlx::pool::PoolConnection<sqlx::Sqlite>;

#[derive(Debug, Clone)]
pub struct Database(SqlitePool);

impl Database {
    #[instrument]
    pub async fn try_from_path<P: AsRef<Path> + Debug>(
        path: P,
        read_only: bool,
    ) -> Result<Self, DatabaseError> {
        tracing::trace!("Creating database pool");
        let res = SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .read_only(read_only),
        )
        .await
        .map(Self)
        .map_err(|e| DatabaseError::PoolSetupError(e.to_string()))?;
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
    pub async fn establish_connection(&self) -> Result<DatabasePoolConnection, DatabaseError> {
        tracing::trace!("Establishing database connection");
        let connection = self
            .0
            .acquire()
            .await
            .map_err(|e| DatabaseError::PoolConnectionError(e.to_string()))?;
        Ok(connection)
    }

    pub async fn run_migrations(&self) -> Result<()> {
        tracing::debug!("Running migrations");
        MIGRATOR
            .run(&self.0)
            .await
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;
        tracing::info!("Database migrated");
        Ok(())
    }
}
