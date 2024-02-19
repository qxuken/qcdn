use clap::Parser;
use tonic::transport::Server;

use qcdn_proto_server::qcdn_general_server::QcdnGeneralServer;

use general::GeneralService;

mod cli;
pub mod constants;
mod files;
mod general;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();

    qcdn_utils::setup_color_eyre()?;

    let cli = cli::Cli::parse();
    cli.instrumentation.setup(&[
        constants::PACKAGE_NAME,
        qcdn_utils::PACKAGE_NAME,
        qcdn_proto_server::PACKAGE_NAME,
        qcdn_database::PACKAGE_NAME,
        qcdn_storage::PACKAGE_NAME,
    ])?;

    let storage = qcdn_storage::Storage::try_from_path(&cli.data).await?;

    let db_path = storage.get_path(qcdn_database::DB_NAME);
    let db = qcdn_database::Database::try_from_path(&db_path).await?;
    db.run_migrations().await?;

    let general = QcdnGeneralServer::new(GeneralService::default());

    tracing::info!("Listening on {}", cli.bind);
    Server::builder()
        .add_service(general)
        .serve(cli.bind)
        .await?;

    Ok(())
}
