use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
    Frame,
};

use super::border_style;
use crate::app::{App, Filter, RepoSelect, SidebarEntry};

pub fn render_sidebar(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    entries: &[SidebarEntry],
    focused: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Groups & Repos ")
        .border_style(border_style(focused))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            entry_item(
                entry,
                entry.filter == app.filter,
                focused && index == app.groups_selected,
            )
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(
        app.groups_selected.min(entries.len().saturating_sub(1)),
    ));

    let list = List::new(items);
    frame.render_stateful_widget(list, inner, &mut state);
}

fn entry_item(entry: &SidebarEntry, active: bool, cursor: bool) -> ListItem<'static> {
    let bar = if cursor {
        Span::styled("▌", Style::new().bold())
    } else {
        Span::raw(" ")
    };
    let marker = if active {
        Span::styled("●", Style::new().cyan())
    } else {
        Span::raw(" ")
    };

    let icon = match entry.filter {
        Filter::All => "≡",
        _ if entry.is_group => "▸",
        _ => "○",
    };

    let label_style = if entry.is_group {
        Style::new().bold()
    } else {
        Style::new()
    };

    let mut spans = vec![
        bar,
        Span::raw(" "),
        marker,
        Span::raw(" "),
        Span::raw(icon),
        Span::raw(" "),
        Span::styled(entry.label.clone(), label_style),
        Span::styled(format!("  {} PR", entry.pr_count), Style::new().dim()),
    ];
    if entry.new_count > 0 {
        spans.push(Span::styled(
            format!(" · {} new", entry.new_count),
            Style::new().fg(Color::Cyan),
        ));
    }

    ListItem::new(Line::from(spans))
}

pub fn render_detail(frame: &mut Frame, area: Rect, app: &App, name: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Group: {name} "))
        .padding(Padding::uniform(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let counts = app.group_repo_counts(name);
    if counts.is_empty() {
        let empty = Paragraph::new("No repositories in this group.").style(Style::new().dim());
        frame.render_widget(empty, inner);
        return;
    }

    let lines: Vec<Line> = counts
        .into_iter()
        .map(|(repo, count, new)| {
            let repo_style = if count > 0 {
                Style::new().fg(Color::Blue)
            } else {
                Style::new().dim()
            };
            let mut spans = vec![
                Span::styled(format!("{repo:<32}"), repo_style),
                Span::styled(format!("  {count} PR"), Style::new().dim()),
            ];
            if new > 0 {
                spans.push(Span::styled(
                    format!(" · {new} new"),
                    Style::new().fg(Color::Cyan),
                ));
            }
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_name_input(frame: &mut Frame, area: Rect, buffer: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" New group · name ")
        .padding(Padding::uniform(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from("Group name:"),
        Line::raw(""),
        Line::from(Span::styled(
            format!("{buffer}▏"),
            Style::new().fg(Color::Cyan).bold(),
        )),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_repo_select(frame: &mut Frame, area: Rect, state: &RepoSelect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" New group: {} · pick repos ", state.name))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.candidates.is_empty() {
        let empty = Paragraph::new("No repositories available to add.").style(Style::new().dim());
        frame.render_widget(empty, inner);
        return;
    }

    let items: Vec<ListItem> = state
        .candidates
        .iter()
        .map(|repo| {
            let mark = if state.chosen.contains(repo) {
                "x"
            } else {
                " "
            };
            ListItem::new(Line::from(format!("[{mark}] {repo}")))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(
        state.cursor.min(state.candidates.len().saturating_sub(1)),
    ));

    let list = List::new(items).highlight_style(Style::new().reversed());
    frame.render_stateful_widget(list, inner, &mut list_state);
}
