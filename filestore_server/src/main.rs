use anyhow::Result;
use clap::Parser;
use config::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::app_state::AppState;
pub use crate::database::{DatabaseConnection, DatabasePoolConnection};
pub use crate::storage::Storage;

mod app_state;
mod config;
mod database;
mod storage;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::parse();

    tracing_subscriber::registry()
        .with(config.log_level)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::trace!(target: "config", "{:?}", config);

    let app_state = AppState::from_config(&config).await?;

    web::run(config, app_state).await?;

    Ok(())
}
