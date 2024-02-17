use clap::Parser;
use std::io::{Error, IsTerminal};

mod cli;

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
    cli.instrumentation.setup(&[env!("CARGO_PKG_NAME")])?;

    tracing::info!("log_level: {}", cli.instrumentation.log_level());
    tracing::info!("{cli:#?}");

    tracing::info!("info");
    tracing::debug!("debug");
    tracing::trace!("trace");
    tracing::error!("error");

    Err(Error::other("throw"))?;

    Ok(())
}
