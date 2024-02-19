use clap::Parser;
use std::io::Error;

mod cli;
mod constants;

fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();

    qcdn_utils::setup_color_eyre()?;

    let cli = cli::Cli::parse();
    cli.instrumentation.setup(&[
        constants::PACKAGE_NAME,
        qcdn_utils::PACKAGE_NAME,
        qcdn_proto_client::PACKAGE_NAME,
    ])?;

    tracing::info!("log_level: {}", cli.instrumentation.log_level());
    tracing::info!("{cli:#?}");

    tracing::info!("info");
    tracing::debug!("debug");
    tracing::trace!("trace");
    tracing::error!("error");

    Err(Error::other("throw"))?;

    Ok(())
}
