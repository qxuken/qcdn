use anyhow::Result;

use crate::{config::Config, database::Database, storage::Storage};

#[derive(Debug, Clone)]
pub struct AppState {
    pub storage: Storage,
    pub db: Database,
}

impl AppState {
    pub async fn from_config(config: &Config) -> Result<Self> {
        let (storage, db) = tokio::try_join!(
            Storage::from_path(&config.storage_dir),
            Database::create_and_migrate(&config.db_path)
        )?;
        let state = Self { storage, db };
        tracing::info!("{:?}", state);
        Ok(state)
    }
}
