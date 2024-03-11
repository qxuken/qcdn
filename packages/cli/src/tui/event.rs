use color_eyre::Result;
use crossterm::event::{self, EventStream, KeyEvent};
use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::{select, sync::mpsc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// An event trigger for render
    Render,
    /// A single key event with additional pressed modifiers.
    Key(KeyEvent),
    /// An resize event with new dimensions after resize (columns, rows).
    Resize(u16, u16),
}

impl Event {
    pub async fn get_event(fps: u64, tx: mpsc::UnboundedSender<Event>) -> Result<()> {
        let mut reader = EventStream::new();
        let render_rate = Duration::from_millis(1_000 / fps);
        let mut render_interval = tokio::time::interval(render_rate);

        loop {
            let render_delay = render_interval.tick();
            let terminal_event = reader.next().fuse();

            select! {
                _ = render_delay => tx.send(Event::Render)?,
                maybe_event = terminal_event => {
                    match maybe_event.transpose()? {
                        Some(event::Event::Key(key_event)) => tx.send(Event::Key(key_event))?,
                        Some(event::Event::Resize(columns, rows)) => tx.send(Event::Resize(columns, rows))?,
                        Some(_) => (),
                        None => break,
                    }
                }
            };
        }
        Ok(())
    }
}
