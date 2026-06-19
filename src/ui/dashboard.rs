use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table},
    Frame,
};

use super::widgets::relative_time;
use crate::app::App;
use crate::github::models::{CiStatus, PullRequest, Urgency};

const SECTIONS: [(Urgency, &str, Color); 3] = [
    (Urgency::Now, "NEEDS ACTION", Color::Red),
    (Urgency::Soon, "SOON", Color::Yellow),
    (Urgency::Background, "BACKGROUND", Color::Gray),
];

pub fn render(app: &App, frame: &mut Frame) {
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header_area, app);
    render_body(frame, body_area, app);
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

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Pull requests ({}) ", app.pull_requests.len()))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.pull_requests.is_empty() {
        let empty = Paragraph::new("No open pull requests authored by you.").dim();
        frame.render_widget(empty, inner);
        return;
    }

    let groups: Vec<(&str, Color, Vec<&PullRequest>)> = SECTIONS
        .iter()
        .filter_map(|(urgency, label, color)| {
            let items: Vec<&PullRequest> = app
                .pull_requests
                .iter()
                .filter(|pr| pr.urgency == *urgency)
                .collect();
            (!items.is_empty()).then_some((*label, *color, items))
        })
        .collect();

    let mut constraints = Vec::new();
    for (_, _, items) in &groups {
        constraints.push(Constraint::Length(1));
        constraints.push(Constraint::Length(items.len() as u16));
        constraints.push(Constraint::Length(1));
    }
    let chunks = Layout::vertical(constraints).split(inner);

    let now = Utc::now();
    for (index, (label, color, items)) in groups.iter().enumerate() {
        let header = Paragraph::new(Line::from(Span::styled(
            format!("● {label} ({})", items.len()),
            Style::new().fg(*color).bold(),
        )));
        frame.render_widget(header, chunks[index * 3]);

        let rows = items.iter().map(|pr| pr_row(pr, now));
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(26),
                Constraint::Min(20),
                Constraint::Length(12),
                Constraint::Length(9),
            ],
        )
        .column_spacing(2);
        frame.render_widget(table, chunks[index * 3 + 1]);
    }
}

fn pr_row<'a>(pr: &'a PullRequest, now: DateTime<Utc>) -> Row<'a> {
    Row::new(vec![
        Cell::from(pr.repo.as_str()),
        Cell::from(Line::from(vec![
            Span::styled(format!("#{} ", pr.number), Style::new().dim()),
            Span::raw(pr.title.as_str()),
        ])),
        Cell::from(ci_badge(pr.ci)),
        Cell::from(Span::styled(
            relative_time(pr.updated_at, now),
            Style::new().dim(),
        )),
    ])
}

fn ci_badge(ci: CiStatus) -> Span<'static> {
    match ci {
        CiStatus::Passing => Span::styled("✓ checks", Style::new().green()),
        CiStatus::Failing => Span::styled("✗ CI failed", Style::new().red()),
        CiStatus::Pending => Span::styled("● running", Style::new().yellow()),
        CiStatus::None => Span::styled("· no CI", Style::new().dim()),
    }
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
