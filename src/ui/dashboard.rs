use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
    Frame,
};

use super::widgets::ci_glyph;
use crate::app::App;
use crate::github::models::{PullRequest, Urgency};

const SECTIONS: [(Urgency, &str, Color); 3] = [
    (Urgency::Now, "NEEDS ACTION", Color::Red),
    (Urgency::Soon, "SOON", Color::Yellow),
    (Urgency::Background, "BACKGROUND", Color::Gray),
];

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Pull requests ({}) ", app.pull_requests.len()))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.pull_requests.is_empty() {
        let (message, style) = empty_message(app);
        frame.render_widget(Paragraph::new(message).style(style), inner);
        return;
    }

    let mut items = Vec::new();
    let mut row_of_pr = vec![0usize; app.pull_requests.len()];
    let mut current: Option<Urgency> = None;

    for (index, pr) in app.pull_requests.iter().enumerate() {
        if current != Some(pr.urgency) {
            current = Some(pr.urgency);
            items.push(section_item(pr.urgency, &app.pull_requests));
        }
        let highlighted = app.highlighted.contains(&(pr.repo.clone(), pr.number));
        row_of_pr[index] = items.len();
        items.push(pr_item(pr, highlighted));
    }

    let mut state = ListState::default();
    state.select(Some(row_of_pr[app.selected]));

    let list = List::new(items).highlight_style(Style::new().reversed());
    frame.render_stateful_widget(list, inner, &mut state);
}

fn empty_message(app: &App) -> (String, Style) {
    if let Some(error) = &app.error {
        (
            format!("Could not load pull requests:\n\n{error}"),
            Style::new().red(),
        )
    } else if app.last_updated.is_none() {
        ("Loading…".to_owned(), Style::new().dim())
    } else {
        (
            "No open pull requests authored by you.".to_owned(),
            Style::new().dim(),
        )
    }
}

fn section_item(urgency: Urgency, pull_requests: &[PullRequest]) -> ListItem<'static> {
    let (label, color) = section_meta(urgency);
    let count = pull_requests
        .iter()
        .filter(|pr| pr.urgency == urgency)
        .count();
    ListItem::new(Line::from(Span::styled(
        format!("● {label} ({count})"),
        Style::new().fg(color).bold(),
    )))
}

fn section_meta(urgency: Urgency) -> (&'static str, Color) {
    SECTIONS
        .iter()
        .find(|(candidate, _, _)| *candidate == urgency)
        .map(|(_, label, color)| (*label, *color))
        .unwrap_or(("", Color::Gray))
}

fn pr_item(pr: &PullRequest, highlighted: bool) -> ListItem<'static> {
    let marker = if highlighted {
        Span::styled("▌", Style::new().cyan().bold())
    } else {
        Span::raw(" ")
    };

    ListItem::new(Line::from(vec![
        marker,
        Span::raw(" "),
        ci_glyph(pr.ci),
        Span::raw(" "),
        Span::styled(format!("{} #{} ", pr.repo, pr.number), Style::new().dim()),
        Span::raw(pr.title.clone()),
    ]))
}
