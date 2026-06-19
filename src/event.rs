use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;

pub enum Action {
    None,
    Quit,
    Back,
    Refresh,
    Next,
    Prev,
    Open,
    Close,
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
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Esc => Action::Back,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Down | KeyCode::Char('j') => Action::Next,
        KeyCode::Up | KeyCode::Char('k') => Action::Prev,
        KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => Action::Open,
        KeyCode::Left | KeyCode::Char('h') => Action::Close,
        _ => Action::None,
    }
}
