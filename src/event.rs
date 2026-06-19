use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;

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
