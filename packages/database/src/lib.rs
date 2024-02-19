use tracing::instrument;

use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub use constants::*;
pub use error::*;
pub use models::*;

pub mod constants;
pub mod error;
pub mod models;
pub(crate) mod schema;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub type DatabaseConnection = PooledConnection<ConnectionManager<diesel::SqliteConnection>>;

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Database {
    #[instrument]
    pub fn try_new(database_url: &str) -> Result<Self, error::DatabaseError> {
        tracing::info!("Creating database connection pool");
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);

        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .map_err(|e| error::DatabaseError::PoolSetupError(e.to_string()))?;

        let db = Database { pool };

        tracing::trace!("{db:#?}");
        Ok(db)
    }
}

impl Database {
    #[instrument(skip(self))]
    pub fn get_connection(&self) -> Result<DatabaseConnection, error::DatabaseError> {
        let pool = self.pool.clone();
        tracing::trace!("Establishing connection");
        pool.get()
            .map_err(|e| error::DatabaseError::PoolConnectionError(e.to_string()))
    }

    #[instrument(skip(self))]
    pub fn run_migrations(&self) -> Result<(), error::DatabaseError> {
        tracing::info!("Running migrations");
        let mut connection = self.get_connection()?;
        connection.run_pending_migrations(MIGRATIONS)?;
        tracing::trace!("Done migrations");

        Ok(())
    }
}
