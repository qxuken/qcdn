use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::app_state::AppState;
pub use crate::database::{DatabaseConnection, DatabasePoolConnection};
pub use crate::storage::Storage;

pub mod app_state;
pub mod config;
pub mod constants;
pub mod database;
pub mod grpc;
pub mod storage;
pub mod web;

pub fn setup_tracing_subscriber(log_level: LevelFilter) {
    tracing_subscriber::registry()
        .with(log_level)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
