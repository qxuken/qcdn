use anyhow::Result;
use listenfd::ListenFd;
use tokio::net::TcpListener;

use crate::{config::CliConfig, AppState};

mod router;

async fn create_listener(config: &CliConfig) -> Result<TcpListener> {
    match ListenFd::from_env().take_tcp_listener(0)? {
        Some(listener) => TcpListener::from_std(listener),
        None => {
            let addr = format!("{}:{}", config.host, config.port);
            TcpListener::bind(addr).await
        }
    }
    .map_err(anyhow::Error::from)
}

pub async fn run(config: &CliConfig, app_state: AppState) -> Result<()> {
    let app = router::create_router().with_state(app_state);

    let listener = create_listener(config).await?;

    tracing::info!("Starting on: http://{}", &listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
