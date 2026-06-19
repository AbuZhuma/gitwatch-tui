mod app;
mod classify;
mod config;
mod event;
mod github;
mod mock;
mod notify;
mod ui;

use std::io::IsTerminal;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use crate::app::{App, Effect};
use crate::github::client::Client;
use crate::github::models::PullRequest;

const REFRESH_SECS: u64 = 30;

#[derive(Parser, Debug)]
#[command(
    name = "gitwatch",
    version,
    about = "Live dashboard for the PRs that need you now."
)]
struct Cli {
    #[arg(long, hide = true)]
    demo: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.demo {
        return run_demo().await;
    }

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
    let repositories = client.repositories().await.unwrap_or_default();
    let groups = config::load();

    let mut terminal = ratatui::init();
    let mut app = App::new(viewer, groups, repositories);
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
            Some(key) = keys.recv() => {
                if matches!(app.handle_key(key.code), Effect::Refresh) {
                    start_refresh(app, &client, &fetch_tx);
                }
            }
        }
    }

    Ok(())
}

async fn run_demo() -> Result<()> {
    let mut app = App::new("octocat".to_owned(), mock::groups(), mock::repos());
    app.set_pull_requests(mock::pull_requests());
    app.highlighted.insert(("acme/web-client".to_owned(), 88));

    let mut terminal = ratatui::init();
    let mut keys = event::spawn_key_reader();
    let result = loop {
        if let Err(e) = terminal.draw(|frame| ui::render(&app, frame)) {
            break Err(e.into());
        }
        if !app.running {
            break Ok(());
        }
        match keys.recv().await {
            Some(key) => {
                app.handle_key(key.code);
            }
            None => break Ok(()),
        }
    };
    ratatui::restore();
    result
}

fn start_refresh(app: &mut App, client: &Arc<Client>, tx: &mpsc::Sender<Result<Vec<PullRequest>>>) {
    if app.refreshing {
        return;
    }
    app.refreshing = true;

    let client = Arc::clone(client);
    let tx = tx.clone();
    let viewer = app.viewer.clone();
    tokio::spawn(async move {
        let _ = tx.send(client.open_pull_requests(&viewer).await).await;
    });
}
