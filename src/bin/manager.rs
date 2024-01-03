use anyhow::Result;
use qcdn::{
    config::CliConfig,
    grpc::{qcdn_general_server::QcdnGeneralServer, server::GeneralService},
    setup_tracing_subscriber,
};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = CliConfig::init();

    setup_tracing_subscriber(config.log_level);

    tracing::debug!("{:?}", config);

    let addr = format!("{}:{}", config.host, config.port).parse()?;

    let general = QcdnGeneralServer::new(GeneralService::default());

    Server::builder().add_service(general).serve(addr).await?;

    Ok(())
}
