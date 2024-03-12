use std::sync::Arc;

use lru::LruCache;
use qcdn_database::Database;
use qcdn_storage::Storage;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub db: Arc<Database>,
    pub files_lru: Arc<Mutex<LruCache<String, i64>>>,
    pub is_dev: bool,
}

pub type SharedAppState = Arc<AppState>;

impl AppState {
    pub fn new(
        storage: Arc<Storage>,
        db: Arc<Database>,
        files_lru: Arc<Mutex<LruCache<String, i64>>>,
        is_dev: bool,
    ) -> Self {
        Self {
            storage,
            db,
            files_lru,
            is_dev,
        }
    }
}
