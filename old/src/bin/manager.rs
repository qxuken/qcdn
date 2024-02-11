use anyhow::Result;
use qcdn::{
    config::CliConfig,
    grpc::{
        qcdn_files_server::QcdnFilesServer,
        qcdn_general_server::QcdnGeneralServer,
        qcdn_nodes_server::QcdnNodesServer,
        server::{files::FilesService, general::GeneralService, nodes::NodesService},
    },
    setup_tracing_subscriber, AppState,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = CliConfig::init();

    setup_tracing_subscriber(config.log_level);

    tracing::debug!("{:?}", config);

    let addr = format!("{}:{}", config.host, config.port).parse()?;
    let app_state = AppState::from_config(&config).await?.shared();
    let (tx, rs) = async_channel::unbounded();

    let general = QcdnGeneralServer::new(GeneralService::default());
    let file = QcdnFilesServer::new(FilesService::new(app_state.clone(), tx));
    let node = QcdnNodesServer::new(NodesService::new(app_state, rs));

    Server::builder()
        .add_service(general)
        .add_service(file)
        .add_service(node)
        .serve(addr)
        .await?;

    Ok(())
}
