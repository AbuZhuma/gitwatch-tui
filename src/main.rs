mod app;
mod classify;
mod config;
mod event;
mod github;
mod notify;
mod ui;

use anyhow::Result;
use clap::Parser;

use crate::app::App;

#[derive(Parser, Debug)]
#[command(
    name = "gitwatch",
    version,
    about = "Live dashboard for the PRs that need you now."
)]
struct Cli {}

fn main() -> Result<()> {
    let _cli = Cli::parse();

    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(app, frame))?;
        event::handle(app)?;
    }
    Ok(())
}
