use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;

pub enum Action {
    None,
    Quit,
    Refresh,
    Next,
    Prev,
}

pub fn spawn_key_reader() -> mpsc::Receiver<KeyEvent> {
    let (tx, rx) = mpsc::channel(32);

    std::thread::spawn(move || loop {
        match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                if tx.blocking_send(key).is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    });

    rx
}

pub fn on_key(code: KeyCode) -> Action {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('j') | KeyCode::Down => Action::Next,
        KeyCode::Char('k') | KeyCode::Up => Action::Prev,
        _ => Action::None,
    }
}
