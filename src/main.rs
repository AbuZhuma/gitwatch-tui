mod app;
mod classify;
mod config;
mod event;
mod github;
mod notify;
mod ui;

use std::io::IsTerminal;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use crate::app::App;
use crate::event::Action;
use crate::github::client::Client;
use crate::github::models::PullRequest;

const REFRESH_SECS: u64 = 30;

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
    let client = Arc::new(Client::new(token)?);
    let viewer = client.viewer_login().await?;

    let mut terminal = ratatui::init();
    let mut app = App::new(viewer);
    let result = run(&mut terminal, &mut app, client).await;
    ratatui::restore();
    result
}

async fn run(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    client: Arc<Client>,
) -> Result<()> {
    let mut keys = event::spawn_key_reader();
    let (fetch_tx, mut fetch_rx) = mpsc::channel::<Result<Vec<PullRequest>>>(4);

    let mut ticker = tokio::time::interval(Duration::from_secs(REFRESH_SECS));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        terminal.draw(|frame| ui::render(app, frame))?;
        if !app.running {
            break;
        }

        tokio::select! {
            _ = ticker.tick() => start_refresh(app, &client, &fetch_tx),
            Some(result) = fetch_rx.recv() => {
                app.refreshing = false;
                match result {
                    Ok(pull_requests) => app.set_pull_requests(pull_requests),
                    Err(e) => app.set_error(format!("{e:#}")),
                }
            }
            Some(key) = keys.recv() => match event::on_key(key.code) {
                Action::Quit => app.quit(),
                Action::Refresh => start_refresh(app, &client, &fetch_tx),
                Action::None => {}
            },
        }
    }

    Ok(())
}

fn start_refresh(app: &mut App, client: &Arc<Client>, tx: &mpsc::Sender<Result<Vec<PullRequest>>>) {
    if app.refreshing {
        return;
    }
    app.refreshing = true;

    let client = Arc::clone(client);
    let tx = tx.clone();
    tokio::spawn(async move {
        let _ = tx.send(client.open_pull_requests().await).await;
    });
}
