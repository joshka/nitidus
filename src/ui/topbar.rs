//! A one-line title bar.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::theme;

pub struct TopBar<'a> {
    pub account: &'a str,
    pub mailbox: &'a str,
    pub busy: Option<&'a str>,
    pub tick: usize,
}

impl Widget for TopBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.bar));
        Line::from(vec![
            Span::styled(
                " nitidus ",
                Style::new()
                    .fg(t.bar)
                    .bg(t.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", self.account), Style::new().fg(t.dim)),
            Span::styled("  ›  ", Style::new().fg(t.faint)),
            Span::styled(self.mailbox.to_string(), Style::new().fg(t.fg)),
        ])
        .render(area, buf);
        if let Some(busy) = self.busy {
            const SPIN: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
            Line::from(vec![
                Span::styled(SPIN[self.tick % SPIN.len()], Style::new().fg(t.accent)),
                Span::styled(format!(" {busy} "), Style::new().fg(t.dim)),
            ])
            .right_aligned()
            .render(area, buf);
        }
    }
}
