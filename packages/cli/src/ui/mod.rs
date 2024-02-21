use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::{io::stdout, time::Duration};

use crate::cli::Cli;

#[derive(Debug, Default)]
struct State {
    key_event: Option<KeyEvent>,
    counter: i64,
    should_quit: bool,
}

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn render(state: &State, frame: &mut Frame) {
    let area = frame.size();
    frame.render_widget(
        Paragraph::new(format!(
            r#"Hello Ratatui! (press 'q' to quit)
Count: {:#?} (j - inc, k - dec)
{:?}
{:?}
"#,
            state.counter, area, state
        ))
        .block(
            Block::default()
                .title("Counter App")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left),
        area,
    );
}

fn update(app: &mut State) -> Result<()> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            app.key_event = Some(key);
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('j') => app.counter += 1,
                    KeyCode::Char('k') => app.counter -= 1,
                    KeyCode::Char('q') => app.should_quit = true,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(stdout()))?;

    // application state
    let mut app = State::default();

    loop {
        // application render
        t.draw(|f| {
            render(&app, f);
        })?;

        // application update
        update(&mut app)?;

        // application exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

pub(crate) async fn ui(_cli: &Cli) -> Result<()> {
    startup()?;
    let status = run();
    shutdown()?;
    status?;
    Ok(())
}
