//! The bottom bar: key hints, or a message that just happened.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::theme;

/// What the bar shows.
pub enum Status<'a> {
    /// Key hints: each key and what it does.
    Hints(&'a [(String, &'static str)]),

    /// Something that succeeded.
    Info(&'a str),

    /// Something that failed, with its causes.
    Error(&'a str),
}

impl Widget for Status<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.bar));
        let line = match self {
            Status::Hints(hints) => hints_line(hints),
            Status::Info(text) => Line::from(vec![
                Span::styled(" ✓ ", Style::new().fg(t.ok)),
                Span::styled(text.to_owned(), Style::new().fg(t.fg)),
            ]),
            Status::Error(text) => Line::from(vec![
                Span::styled(" ✗ ", Style::new().fg(t.bar).bg(t.error)),
                Span::styled(format!(" {text}"), Style::new().fg(t.error)),
            ]),
        };
        line.render(area, buf);
    }
}

/// Each key as a chip, then what it does.
fn hints_line(hints: &[(String, &'static str)]) -> Line<'static> {
    let t = theme();
    let hint = |(key, text): &(String, &'static str)| {
        [
            Span::styled(format!(" {key} "), Style::new().fg(t.fg).bg(t.chip)),
            Span::styled(format!(" {text}  "), Style::new().fg(t.dim)),
        ]
    };
    Line::from(
        std::iter::once(Span::raw(" "))
            .chain(hints.iter().flat_map(hint))
            .collect::<Vec<_>>(),
    )
}
