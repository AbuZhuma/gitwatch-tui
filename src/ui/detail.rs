use chrono::{DateTime, Utc};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use super::widgets::{ci_glyph, relative_time};
use crate::app::App;
use crate::github::models::{
    Activity, Check, MergeState, PullRequest, Review, ReviewState, Urgency,
};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Details ")
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible = app.visible();
    let Some(pr) = visible.get(app.selected) else {
        let placeholder = Paragraph::new("Select a pull request.").style(Style::new().dim());
        frame.render_widget(placeholder, inner);
        return;
    };

    let now = Utc::now();
    let mut lines = vec![
        Line::from(Span::styled(pr.title.clone(), Style::new().bold())),
        Line::from(Span::styled(
            format!("{} #{}", pr.repo, pr.number),
            Style::new().dim(),
        )),
        Line::raw(""),
        Line::from(format!("{} → {}", pr.head_ref, pr.base_ref)),
        status_line(pr),
        Line::raw(""),
    ];

    lines.push(Line::from(Span::styled("Checks", Style::new().bold())));
    if pr.checks.is_empty() {
        lines.push(dim_line("  no checks"));
    } else {
        for check in &pr.checks {
            lines.push(check_line(check));
        }
    }
    lines.push(Line::raw(""));

    lines.push(Line::from(Span::styled("Reviews", Style::new().bold())));
    if pr.reviews.is_empty() {
        lines.push(dim_line("  no reviews yet"));
    } else {
        for review in &pr.reviews {
            lines.push(review_line(review));
        }
    }
    lines.push(Line::raw(""));

    lines.push(Line::from(Span::styled(
        "Recent activity",
        Style::new().bold(),
    )));
    if pr.activity.is_empty() {
        lines.push(dim_line("  no comments"));
    } else {
        for item in pr.activity.iter().rev().take(5) {
            lines.push(activity_line(item, now));
        }
    }

    let details = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(details, inner);
}

fn status_line(pr: &PullRequest) -> Line<'static> {
    let (label, color) = urgency_meta(pr.urgency);
    let merge = match pr.mergeable {
        MergeState::Mergeable => Span::styled("mergeable", Style::new().green()),
        MergeState::Conflicting => Span::styled("conflict", Style::new().red()),
        MergeState::Unknown => Span::styled("checking…", Style::new().dim()),
    };

    let mut spans = vec![
        Span::styled(format!("● {label}"), Style::new().fg(color).bold()),
        Span::raw("  "),
        merge,
    ];
    if pr.is_draft {
        spans.push(Span::raw("  "));
        spans.push(Span::styled("draft", Style::new().dim()));
    }
    Line::from(spans)
}

fn urgency_meta(urgency: Urgency) -> (&'static str, Color) {
    match urgency {
        Urgency::Now => ("needs action", Color::Red),
        Urgency::Soon => ("soon", Color::Yellow),
        Urgency::Background => ("background", Color::Gray),
    }
}

fn check_line(check: &Check) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        ci_glyph(check.state),
        Span::raw(" "),
        Span::raw(check.name.clone()),
    ])
}

fn review_line(review: &Review) -> Line<'static> {
    let (glyph, label) = match review.state {
        ReviewState::Approved => (Span::styled("✓", Style::new().green()), "approved"),
        ReviewState::ChangesRequested => {
            (Span::styled("✗", Style::new().red()), "changes requested")
        }
        ReviewState::Commented => (Span::styled("●", Style::new().yellow()), "commented"),
        ReviewState::Other => (Span::styled("·", Style::new().dim()), "reviewed"),
    };

    Line::from(vec![
        Span::raw("  "),
        glyph,
        Span::raw(" "),
        Span::raw(review.login.clone()),
        Span::styled(format!("  {label}"), Style::new().dim()),
    ])
}

fn activity_line(item: &Activity, now: DateTime<Utc>) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{} ", item.author), Style::new().bold()),
        Span::styled(relative_time(item.at, now), Style::new().dim()),
        Span::raw(format!(": {}", item.summary)),
    ])
}

fn dim_line(text: &str) -> Line<'static> {
    Line::from(Span::styled(text.to_owned(), Style::new().dim()))
}
