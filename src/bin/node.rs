use qcdn::{
    config::CliConfig,
    grpc::{
        qcdn_files_client::QcdnFilesClient, qcdn_general_client::QcdnGeneralClient,
        qcdn_nodes_client::QcdnNodesClient,
    },
    setup_tracing_subscriber, AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = CliConfig::init();

    setup_tracing_subscriber(config.log_level);

    tracing::info!("{:?}", config);

    let _app_state = AppState::from_config(&config).await?.shared();

    let addr = config
        .main_server_url
        .expect("Master address must be present");

    let _general = QcdnGeneralClient::connect(addr.clone()).await?;

    let _files = QcdnFilesClient::connect(addr.clone()).await?;

    let _nodes = QcdnNodesClient::connect(addr).await?;

    Ok(())
}
