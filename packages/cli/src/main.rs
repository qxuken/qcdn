use clap::Parser;
use color_eyre::Result;

pub(crate) mod cli;
pub mod constants;
mod ui;
mod upload;

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

    match &cli.command {
        cli::Command::Ui => ui::ui(&cli).await,
        cli::Command::Upload {
            version,
            save_version,
        } => upload::upload(&cli, version, save_version).await,
    }?;

    Ok(())
}
