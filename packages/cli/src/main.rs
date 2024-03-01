use clap::Parser;
use color_eyre::Result;

pub mod cli;
mod commands;
pub mod constants;
mod rpc;
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

    match cli.command.clone().unwrap_or_default() {
        cli::Command::Connect => commands::handshake_server::handshake_server(&cli).await,
        cli::Command::Dirs { format } => commands::list_dirs::list_dirs(&cli, format).await,
        cli::Command::Files { dir_id, format } => {
            commands::list_files::list_files(&cli, dir_id, format).await
        }
        cli::Command::Versions { file_id, format } => {
            commands::list_versions::list_versions(&cli, file_id, format).await
        }
        cli::Command::Download {
            file_version_id,
            path,
        } => commands::download::download(&cli, file_version_id, path).await,
        cli::Command::Upload {
            media_type,
            dir,
            name,
            version,
            src,
        } => commands::upload::upload(&cli, dir, name, version, media_type, src).await,
        cli::Command::Tag {
            file_version_id,
            tag,
        } => commands::tag_version::tag_version(&cli, file_version_id, tag).await,
        cli::Command::Delete { file_version_id } => {
            commands::delete_version::delete_version(&cli, file_version_id).await
        }
    }?;

    Ok(())
}
