mod app;
mod classify;
mod config;
mod event;
mod github;
mod notify;
mod ui;

use std::io::IsTerminal;

use anyhow::Result;
use clap::Parser;

use crate::app::App;
use crate::github::client::Client;

#[derive(Parser, Debug)]
#[command(
    name = "gitwatch",
    version,
    about = "Live dashboard for the PRs that need you now."
)]
struct Cli {}

#[tokio::main]
async fn main() -> Result<()> {
    let _cli = Cli::parse();

    let token = match github::auth::token() {
        Ok(token) => token,
        Err(e) => {
            let color = std::io::stderr().is_terminal();
            eprint!("{}", e.guidance().render(color));
            std::process::exit(1);
        }
    };
    let client = Client::new(token)?;
    let viewer = client.viewer_login().await?;
    let pull_requests = client.open_pull_requests().await?;

    let mut terminal = ratatui::init();
    let mut app = App::new(viewer, pull_requests);
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
