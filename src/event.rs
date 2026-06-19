use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::App;

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

pub fn handle(app: &mut App) -> Result<()> {
    if event::poll(POLL_TIMEOUT)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                on_key(app, key.code);
            }
        }
    }
    Ok(())
}

fn on_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        _ => {}
    }
}
