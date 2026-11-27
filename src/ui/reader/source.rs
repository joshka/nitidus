//! The raw source view: header names highlighted, everything else as written.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::render_scrolled;
use crate::ui::layout::{below, line};
use crate::ui::theme;

/// Draws `source`, or a loading note while it is read.
pub fn render(source: Option<&str>, scroll: &mut u16, area: Rect, buf: &mut Buffer) {
    let t = theme();
    let inner = area.inner(Margin::new(3, 1));
    let title = Style::new().fg(t.accent).add_modifier(Modifier::BOLD);
    Line::styled("Message source", title).render(line(inner, 0), buf);
    let lines = source
        .unwrap_or("Loading source…")
        .lines()
        .map(source_line)
        .collect();
    render_scrolled(lines, scroll, below(inner, 2), buf);
}

/// A header line with its name highlighted; any other line dim.
fn source_line(text: &str) -> Line<'static> {
    let t = theme();
    match text.split_once(':') {
        Some((name, value)) if !name.is_empty() && !name.contains(' ') => Line::from(vec![
            Span::styled(name.to_owned(), Style::new().fg(t.accent)),
            Span::styled(format!(":{value}"), Style::new().fg(t.dim)),
        ]),
        _ => Line::styled(text.to_owned(), Style::new().fg(t.dim)),
    }
}
