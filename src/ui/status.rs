//! The bottom bar: key hints, a transient message, or the search prompt.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::theme;

pub enum Status<'a> {
    Hints(&'a [(&'a str, &'a str)]),
    Info(&'a str),
    Error(&'a str),
}

impl Widget for Status<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.bar));
        let line = match self {
            Status::Hints(hints) => {
                let mut spans = vec![Span::raw(" ")];
                for (key, what) in hints {
                    spans.push(Span::styled(
                        format!(" {key} "),
                        Style::new().fg(t.fg).bg(t.chip),
                    ));
                    spans.push(Span::styled(format!(" {what}  "), Style::new().fg(t.dim)));
                }
                Line::from(spans)
            }
            Status::Info(msg) => Line::from(vec![
                Span::styled(" ✓ ", Style::new().fg(t.ok)),
                Span::styled(msg.to_string(), Style::new().fg(t.fg)),
            ]),
            Status::Error(msg) => Line::from(vec![
                Span::styled(" ✗ ", Style::new().fg(t.bar).bg(t.error)),
                Span::styled(format!(" {msg}"), Style::new().fg(t.error)),
            ]),
        };
        line.render(area, buf);
    }
}
