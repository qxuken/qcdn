use std::time::Duration;

use color_eyre::Result;
use crossterm::{
    cursor::position,
    event::{Event, EventStream, KeyCode},
};
use futures::{FutureExt, StreamExt};
use ratatui::Frame;
use tokio::{select, sync::mpsc};

use crate::{cli::Cli, rpc::Rpc};

pub mod errors;
pub mod event;
pub mod term;

async fn print_events() {
    let mut reader = EventStream::new();
    let tick_rate = Duration::from_millis(250);
    let mut interval = tokio::time::interval(tick_rate);

    loop {
        let delay = interval.tick();
        let event = reader.next().fuse();

        select! {
            _ = delay => { println!(".\r"); },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        println!("Event::{:?}\r", event);

                        if event == Event::Key(KeyCode::Char('c').into()) {
                            println!("Cursor position: {:?}\r", position());
                        }

                        if event == Event::Key(KeyCode::Esc.into()) {
                            break;
                        }
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        };
    }
}

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
