use std::time::SystemTime;

use qcdn::{
    config::CliConfig,
    grpc::{qcdn_general_client::QcdnGeneralClient, PingMessage},
    setup_tracing_subscriber,
};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = CliConfig::init();

    setup_tracing_subscriber(config.log_level);

    tracing::debug!("{:?}", config);

    let addr = config
        .main_server_url
        .expect("Master address must be present");

    let mut general = QcdnGeneralClient::connect(addr).await?;

    let ping = PingMessage {
        timestamp: Some(SystemTime::now().into()),
    };
    let response = general.ping(Request::new(ping)).await?.into_inner();

    println!("{response:?}");

    let response = general.version(Request::new(())).await?.into_inner();

    println!("{response:?}");

    Ok(())
}
