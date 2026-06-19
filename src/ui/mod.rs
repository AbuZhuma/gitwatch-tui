mod dashboard;
mod detail;
mod widgets;

use chrono::Utc;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use widgets::relative_time;

pub fn render(app: &App, frame: &mut Frame) {
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header_area, app);

    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .areas(body_area);
    dashboard::render(frame, list_area, app);
    detail::render(frame, detail_area, app);

    render_footer(frame, footer_area);
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

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::raw(" q ").reversed(),
        Span::raw(" quit"),
        Span::raw("  "),
        Span::raw(" j/k ").reversed(),
        Span::raw(" move"),
        Span::raw("  "),
        Span::raw(" r ").reversed(),
        Span::raw(" refresh"),
        Span::raw("  "),
        Span::raw(" ? ").reversed(),
        Span::raw(" help"),
    ]))
    .style(Style::new().dim());
    frame.render_widget(footer, area);
}
