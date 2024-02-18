use clap::Parser;
use std::io::{Error, IsTerminal};

mod cli;
mod constants;

fn main() -> color_eyre::Result<()> {
    color_eyre::config::HookBuilder::default()
        .theme(if !std::io::stderr().is_terminal() {
            // Don't attempt color
            color_eyre::config::Theme::new()
        } else {
            color_eyre::config::Theme::dark()
        })
        .install()?;

    let cli = cli::Cli::parse();
    cli.instrumentation.setup(&[constants::PACKAGE_NAME, qcdn_proto_client::PACKAGE_NAME])?;

    tracing::info!("log_level: {}", cli.instrumentation.log_level());
    tracing::info!("{cli:#?}");

    tracing::info!("info");
    tracing::debug!("debug");
    tracing::trace!("trace");
    tracing::error!("error");

    Err(Error::other("throw"))?;

    Ok(())
}
