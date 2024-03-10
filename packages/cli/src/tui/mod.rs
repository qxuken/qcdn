use color_eyre::Result;
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::{cli::Cli, rpc::Rpc};

pub mod errors;
pub mod term;

pub trait Screen {
    type Action;

    fn handle(self, action: Self::Action, tx: mpsc::UnboundedSender<Self::Action>);
    fn render(self, frame: Frame);
}

#[tracing::instrument(skip_all)]
pub async fn run(cli: &Cli) -> Result<()> {
    let rpc: Rpc = cli.into();

    let client_version = Rpc::client_version();
    let general = rpc.connect_to_general().await?;
    let (latency, server_version) = tokio::try_join!(
        async {
            let mut general = general.clone();
            Rpc::latency(&mut general).await
        },
        async {
            let mut general = general.clone();
            Rpc::server_version(&mut general).await
        }
    )?;

    if let Some(latency) = latency {
        println!("Latency to server is {}", latency);
    }

    if client_version != server_version {
        println!(
            "Version mismatch!\nServer version is {}, client version is {}",
            server_version,
            Rpc::client_version()
        );
    } else {
        println!("Server version is {}", server_version,);
    }

    Ok(())
}
