use tracing::instrument;

use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    SqliteConnection,
};

pub use error::*;
pub use models::*;

pub mod error;
pub mod models;
pub(crate) mod schema;

pub type DatabaseConnection = PooledConnection<ConnectionManager<diesel::SqliteConnection>>;

#[derive(Debug, Clone)]
pub struct Database {
    db: Pool<ConnectionManager<SqliteConnection>>,
}

impl Database {
    #[instrument]
    pub fn try_new(database_url: &str) -> Result<Self, error::DatabaseError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);

        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .map_err(|e| error::DatabaseError::PoolSetupError(e.to_string()))?;

        Ok(Database { db: pool })
    }
}

impl Database {
    pub fn get_connection(&self) -> Result<DatabaseConnection, error::DatabaseError> {
        let pool = self.db.clone();

        pool.get()
            .map_err(|e| error::DatabaseError::PoolConnectionError(e.to_string()))
    }
}
