use chrono::{DateTime, Utc};
use ratatui::{
    style::{Style, Stylize},
    text::Span,
};

use crate::github::models::CiStatus;

pub fn relative_time(time: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = (now - time).num_seconds();
    if seconds < 0 {
        return "just now".to_owned();
    }

    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if seconds < 60 {
        format!("{seconds}s ago")
    } else if minutes < 60 {
        format!("{minutes}m ago")
    } else if hours < 24 {
        format!("{hours}h ago")
    } else if days < 7 {
        format!("{days}d ago")
    } else if days < 30 {
        format!("{}w ago", days / 7)
    } else if days < 365 {
        format!("{}mo ago", days / 30)
    } else {
        format!("{}y ago", days / 365)
    }
}

pub fn ci_glyph(ci: CiStatus) -> Span<'static> {
    match ci {
        CiStatus::Passing => Span::styled("✓", Style::new().green()),
        CiStatus::Failing => Span::styled("✗", Style::new().red()),
        CiStatus::Pending => Span::styled("●", Style::new().yellow()),
        CiStatus::None => Span::styled("·", Style::new().dim()),
    }
}
