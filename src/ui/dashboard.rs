use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame) {
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header_area, app);
    render_body(frame, body_area);
    render_footer(frame, footer_area);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let header = Paragraph::new(Line::from(vec![
        Span::raw(" gitwatch "),
        Span::raw(format!("· @{} ", app.viewer)),
    ]))
    .style(Style::new().reversed().bold());
    frame.render_widget(header, area);
}

fn render_body(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Dashboard ")
        .padding(Padding::uniform(1));

    let lines = vec![
        Line::from("Signed in.").bold(),
        Line::from(""),
        Line::from("Next stages:"),
        Line::from("  • stage 2 — fetch your open PRs (GraphQL batch)"),
        Line::from("  • stage 3 — urgency sections 🔴 🟡 ⚪"),
        Line::from("  • stage 5 — split-pane PR details"),
    ];

    let body = Paragraph::new(lines).block(block);
    frame.render_widget(body, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::raw(" q ").reversed(),
        Span::raw(" quit"),
        Span::raw("   "),
        Span::raw(" ? ").reversed(),
        Span::raw(" help (soon)"),
    ]))
    .style(Style::new().dim());
    frame.render_widget(footer, area);
}
