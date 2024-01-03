use anyhow::Result;
use qcdn::{config::CliConfig, setup_tracing_subscriber, web, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = CliConfig::init();

    setup_tracing_subscriber(config.log_level);

    tracing::debug!("{:?}", config);

    let app_state = AppState::from_config(&config).await?;

    web::run(&config, app_state).await?;

    Ok(())
}
