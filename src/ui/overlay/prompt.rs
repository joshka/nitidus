//! A line of text input, for searches, names and paths.

use ratatui::crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::{Edit, Outcome, draw_panel, panel_area};
use crate::ui::layout::line;
use crate::ui::theme;

/// The panel's size.
const SIZE: (u16, u16) = (56, 7);

/// A one-line text field with a hint below it.
pub struct Prompt {
    /// The panel's title.
    title: String,

    /// The text typed so far.
    input: String,

    /// A faint hint under the field.
    hint: String,
}

impl Prompt {
    /// A prompt titled `title`, starting with `input`, hinting `hint`.
    pub fn new(
        title: impl Into<String>,
        input: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            input: input.into(),
            hint: hint.into(),
        }
    }

    /// Applies a key: typing edits, Enter answers the trimmed text, Esc cancels.
    pub fn handle(&mut self, key: KeyEvent) -> Outcome<String> {
        match Edit::of(key) {
            Some(Edit::Cancel) => Outcome::Cancelled,
            Some(Edit::Submit) => Outcome::Done(self.input.trim().to_owned()),
            Some(Edit::Erase) => {
                self.input.pop();
                Outcome::Open
            }
            Some(Edit::Type(c)) => {
                self.input.push(c);
                Outcome::Open
            }
            Some(Edit::Next | Edit::Previous) | None => Outcome::Open,
        }
    }
}

impl Widget for &Prompt {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let inner = draw_panel(panel_area(area, SIZE.0, SIZE.1), &self.title, buf);
        let field = line(inner, 0);
        buf.set_style(field, Style::new().bg(t.bg));
        let text = vec![
            Span::styled(format!(" {}", self.input), Style::new().fg(t.fg)),
            Span::styled("▏", Style::new().fg(t.accent)),
        ];
        Line::from(text).render(field, buf);
        Line::styled(self.hint.clone(), Style::new().fg(t.faint)).render(line(inner, 2), buf);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::crossterm::event::KeyCode;

    use super::*;
    use crate::ui::test_support::render;

    #[test]
    fn typing_edits_and_enter_answers_the_trimmed_text() {
        let mut prompt = Prompt::new("Search", "from:", "hint");
        for key in [
            KeyCode::Char('a'),
            KeyCode::Char('x'),
            KeyCode::Backspace,
            KeyCode::Char(' '),
        ] {
            prompt.handle(KeyEvent::from(key));
        }

        let answer = prompt.handle(KeyEvent::from(KeyCode::Enter));

        assert!(matches!(answer, Outcome::Done(text) if text == "from:a"));
    }

    #[test]
    fn the_field_shows_the_text_with_a_cursor_and_the_hint_below() {
        let prompt = Prompt::new("New mailbox", "Receipts", "Name of the mailbox to create");

        let screen = render(60, 9, |area, buf| (&prompt).render(area, buf));

        assert_snapshot!(screen, @"
        New mailbox

         Receipts▏

        Name of the mailbox to create
        ");
    }
}
