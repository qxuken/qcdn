use clap::Parser;
use color_eyre::Result;
use rpc::Rpc;

pub mod cli;
mod commands;
pub mod constants;
mod rpc;
mod tui;
pub mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    qcdn_utils::setup_color_eyre()?;

    let cli = cli::Cli::parse();
    cli.instrumentation.setup(&[
        constants::PACKAGE_NAME,
        qcdn_utils::PACKAGE_NAME,
        qcdn_proto_client::PACKAGE_NAME,
    ])?;

    let rpc: Rpc = (&cli).into();

    match cli.command.clone().unwrap_or_default() {
        cli::Command::Ui => tui::ui().await,
        cli::Command::Connect => commands::handshake_server::handshake_server(&rpc).await,
        cli::Command::Dirs { format } => commands::list_dirs::list_dirs(&rpc, format).await,
        cli::Command::Files { dir_id, format } => {
            commands::list_files::list_files(&rpc, dir_id, format).await
        }
        cli::Command::Versions { file_id, format } => {
            commands::list_versions::list_versions(&rpc, file_id, format).await
        }
        cli::Command::Upload {
            version,
            save_version,
        } => commands::upload::upload(&cli, &version, save_version).await,
    }?;

    Ok(())
}
