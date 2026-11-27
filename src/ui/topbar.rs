//! The title bar: the app, the account and the open mailbox, with a spinner while work runs.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::theme;

/// The spinner's frames.
const SPINNER: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

/// The title bar.
pub struct TopBar<'a> {
    /// The account's name.
    pub account: &'a str,

    /// What the main area shows: the mailbox, or "Compose".
    pub place: &'a str,

    /// What is running, if anything.
    pub busy: Option<&'a str>,

    /// The frame count, which turns the spinner.
    pub tick: usize,
}

impl Widget for TopBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.bar));
        Line::from(self.title()).render(area, buf);
        if let Some(busy) = self.busy {
            let spinner = Span::styled(
                SPINNER[self.tick % SPINNER.len()],
                Style::new().fg(t.accent),
            );
            let busy = Span::styled(format!(" {busy} "), Style::new().fg(t.dim));
            Line::from(vec![spinner, busy])
                .right_aligned()
                .render(area, buf);
        }
    }
}

impl TopBar<'_> {
    /// The app's badge, the account and where the main area is.
    fn title(&self) -> Vec<Span<'static>> {
        let t = theme();
        let badge = Style::new()
            .fg(t.bar)
            .bg(t.accent)
            .add_modifier(Modifier::BOLD);
        vec![
            Span::styled(" nitidus ", badge),
            Span::styled(format!("  {}", self.account), Style::new().fg(t.dim)),
            Span::styled("  ›  ", Style::new().fg(t.faint)),
            Span::styled(self.place.to_owned(), Style::new().fg(t.fg)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::ui::test_support::render;

    #[test]
    fn the_title_names_the_place_and_what_is_running() {
        let bar = TopBar {
            account: "demo",
            place: "INBOX",
            busy: Some("Reading"),
            tick: 0,
        };

        let screen = render(50, 1, |area, buf| bar.render(area, buf));

        assert_snapshot!(screen, @" nitidus   demo  ›  INBOX               ⣾ Reading");
    }
}
