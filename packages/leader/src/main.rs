#![feature(try_blocks)]

use std::sync::Arc;

use clap::Parser;
use tonic::transport::Server;

use qcdn_proto_server::{
    qcdn_file_queries_server::QcdnFileQueriesServer,
    qcdn_file_updates_server::QcdnFileUpdatesServer, qcdn_general_server::QcdnGeneralServer,
};

use general::GeneralService;

use crate::file::FileService;

mod cli;
pub mod constants;
mod file;
mod general;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();

    qcdn_utils::color_eyre::setup()?;

    let cli = cli::Cli::parse();
    cli.instrumentation.setup(&[
        constants::PACKAGE_NAME,
        qcdn_utils::PACKAGE_NAME,
        qcdn_proto_server::PACKAGE_NAME,
        qcdn_database::PACKAGE_NAME,
        qcdn_storage::PACKAGE_NAME,
    ])?;

    let storage = Arc::new(qcdn_storage::Storage::try_from_path(&cli.data, "storage").await?);

    let db_path = storage.get_path(qcdn_database::DB_NAME, true);
    let db = Arc::new(qcdn_database::Database::try_from_path(&db_path).await?);
    db.run_migrations().await?;

    let general = QcdnGeneralServer::new(GeneralService::default());
    let files_queries = QcdnFileQueriesServer::new(FileService::new(storage.clone(), db.clone()));
    let files_updates = QcdnFileUpdatesServer::new(FileService::new(storage.clone(), db.clone()));

    tracing::info!("Listening on {}", cli.bind);
    Server::builder()
        .add_service(general)
        .add_service(files_queries)
        .add_service(files_updates)
        .serve(cli.bind)
        .await?;

    Ok(())
}
