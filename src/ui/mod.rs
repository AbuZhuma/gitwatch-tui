mod dashboard;
mod detail;
mod groups;
mod widgets;

use chrono::Utc;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, Focus, Mode};
use widgets::relative_time;

pub fn render(app: &App, frame: &mut Frame) {
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header_area, app);

    match &app.mode {
        Mode::Browse => render_browse(frame, body_area, app),
        Mode::GroupDetail(name) => groups::render_detail(frame, body_area, app, name),
        Mode::NameInput(buffer) => groups::render_name_input(frame, body_area, buffer),
        Mode::RepoSelect(state) => groups::render_repo_select(frame, body_area, state),
    }

    render_footer(frame, footer_area, app);
}

fn render_browse(frame: &mut Frame, area: Rect, app: &App) {
    let entries = app.sidebar_entries();
    let groups_height = (entries.len() as u16 + 2).clamp(3, (area.height / 2).max(3));

    let [groups_area, pr_area] =
        Layout::vertical([Constraint::Length(groups_height), Constraint::Min(0)]).areas(area);

    groups::render_sidebar(
        frame,
        groups_area,
        app,
        &entries,
        app.focus == Focus::Groups,
    );

    let list_focused = app.focus == Focus::List;
    if app.detail_open && !app.visible().is_empty() {
        let [list_area, detail_area] =
            Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                .areas(pr_area);
        dashboard::render(frame, list_area, app, list_focused);
        detail::render(frame, detail_area, app);
    } else {
        dashboard::render(frame, pr_area, app, list_focused);
    }
}

pub(super) fn border_style(focused: bool) -> Style {
    if focused {
        Style::new().fg(Color::Cyan)
    } else {
        Style::new().dim()
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let left = format!(" gitwatch · @{} ", app.viewer);
    let right = format!("{} ", status_text(app));

    let width = area.width as usize;
    let used = left.chars().count() + right.chars().count();
    let pad = " ".repeat(width.saturating_sub(used));

    let header =
        Paragraph::new(format!("{left}{pad}{right}")).style(Style::new().reversed().bold());
    frame.render_widget(header, area);
}

fn status_text(app: &App) -> String {
    if app.refreshing {
        return "⟳ refreshing…".to_owned();
    }
    if app.error.is_some() {
        return match app.last_updated {
            Some(at) => format!("⚠ update failed · last {}", relative_time(at, Utc::now())),
            None => "⚠ update failed".to_owned(),
        };
    }
    match app.last_updated {
        Some(at) => {
            let mut text = format!("⟳ updated {}", relative_time(at, Utc::now()));
            if !app.highlighted.is_empty() {
                text.push_str(&format!(" · {} new", app.highlighted.len()));
            }
            text
        }
        None => String::new(),
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let hints: &[(&str, &str)] = match &app.mode {
        Mode::Browse => match app.focus {
            Focus::Groups => &[
                ("↑↓", "move"),
                ("↵", "filter"),
                ("→", "details"),
                ("n", "new group"),
                ("q", "quit"),
            ],
            Focus::List => &[
                ("↑↓", "move"),
                ("→", "open"),
                ("←", "back"),
                ("r", "refresh"),
                ("q", "quit"),
            ],
        },
        Mode::GroupDetail(_) => &[("←", "back"), ("q", "quit")],
        Mode::NameInput(_) => &[("type", "name"), ("↵", "next"), ("esc", "cancel")],
        Mode::RepoSelect(_) => &[
            ("↑↓", "move"),
            ("space", "pick"),
            ("↵", "create"),
            ("esc", "cancel"),
        ],
    };

    let mut spans = Vec::new();
    for (key, label) in hints {
        spans.push(Span::raw(format!(" {key} ")).reversed());
        spans.push(Span::raw(format!(" {label}  ")));
    }

    let footer = Paragraph::new(Line::from(spans)).style(Style::new().dim());
    frame.render_widget(footer, area);
}
