use std::sync::Arc;

use qcdn_database::Database;
use qcdn_storage::Storage;

#[derive(Debug, Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub db: Arc<Database>,
    pub is_dev: bool,
}

pub type SharedAppState = Arc<AppState>;

impl AppState {
    pub fn new(storage: Arc<Storage>, db: Arc<Database>, is_dev: bool) -> Self {
        Self {
            storage,
            db,
            is_dev,
        }
    }
}
