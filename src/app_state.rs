use std::sync::Arc;

use anyhow::Result;

use crate::{config::CliConfig, database::Database, storage::Storage};

#[derive(Debug, Clone)]
pub struct AppState {
    pub storage: Storage,
    pub db: Database,
    pub config: Arc<CliConfig>,
}

impl AppState {
    pub async fn from_config(config: &CliConfig) -> Result<Self> {
        let (storage, db) = tokio::try_join!(
            Storage::from_path(&config.storage_dir),
            Database::create_and_migrate(&config.db_path)
        )?;
        let state = Self {
            storage,
            db,
            config: Arc::new(config.clone()),
        };
        tracing::info!("{:?}", state);
        Ok(state)
    }
}

impl AppState {
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}
