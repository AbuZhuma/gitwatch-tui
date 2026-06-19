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

    let items: Vec<ListItem> = app
        .pull_requests
        .iter()
        .enumerate()
        .map(|(index, pr)| {
            let highlighted = app.highlighted.contains(&(pr.repo.clone(), pr.number));
            pr_item(pr, highlighted, index == app.selected)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let list = List::new(items);
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

fn pr_item(pr: &PullRequest, highlighted: bool, selected: bool) -> ListItem<'static> {
    let (label, color) = priority(pr.urgency);

    let cursor = if selected {
        Span::styled("▌", Style::new().bold())
    } else {
        Span::raw(" ")
    };

    let base = Style::new().fg(color);
    let text_style = if highlighted { base.bold() } else { base };

    ListItem::new(Line::from(vec![
        cursor,
        Span::raw(" "),
        Span::styled(format!("{label:<4}"), base.bold()),
        Span::raw("  "),
        ci_glyph(pr.ci),
        Span::raw(" "),
        Span::styled(format!("{} #{} ", pr.repo, pr.number), text_style),
        Span::styled(pr.title.clone(), text_style),
    ]))
}

fn priority(urgency: Urgency) -> (&'static str, Color) {
    match urgency {
        Urgency::Now => ("NOW", Color::Red),
        Urgency::Soon => ("SOON", Color::Yellow),
        Urgency::Background => ("LOW", Color::Cyan),
    }
}
