use anyhow::Result;
use qcdn::{
    config::CliConfig,
    grpc::{
        qcdn_files_server::QcdnFilesServer,
        qcdn_general_server::QcdnGeneralServer,
        server::{files::FilesService, general::GeneralService},
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
    let app_state = AppState::from_config(&config).await?;

    let general = QcdnGeneralServer::new(GeneralService::default());
    let file = QcdnFilesServer::new(FilesService::new(app_state));

    Server::builder()
        .add_service(general)
        .add_service(file)
        .serve(addr)
        .await?;

    Ok(())
}
