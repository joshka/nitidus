//! A yes/no question before something that can't be undone.

use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::{Outcome, draw_panel, panel_area};
use crate::ui::layout::{cells, line};
use crate::ui::theme;

/// The narrowest the panel is.
const MIN_WIDTH: u16 = 36;

/// A yes/no question.
pub struct Confirm {
    /// The question.
    question: String,
}

impl Confirm {
    /// Asks `question`.
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
        }
    }

    /// `y` answers yes; `n`, `q` or Esc no; other keys are ignored.
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "a question reads three keys and ignores the rest"
    )]
    pub fn handle(key: KeyEvent) -> Outcome<()> {
        match key.code {
            KeyCode::Char('y' | 'Y') => Outcome::Done(()),
            KeyCode::Char('n' | 'q') | KeyCode::Esc => Outcome::Cancelled,
            _ => Outcome::Open,
        }
    }
}

impl Widget for &Confirm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let width = (cells(self.question.chars().count()) + 8).max(MIN_WIDTH);
        let inner = draw_panel(panel_area(area, width, 6), "Are you sure?", buf);
        Line::styled(self.question.clone(), Style::new().fg(t.fg)).render(line(inner, 0), buf);
        let choices = vec![
            Span::styled(" y ", Style::new().fg(t.fg).bg(t.chip)),
            Span::styled(" yes   ", Style::new().fg(t.dim)),
            Span::styled(" n ", Style::new().fg(t.fg).bg(t.chip)),
            Span::styled(" no", Style::new().fg(t.dim)),
        ];
        Line::from(choices).render(line(inner, 2), buf);
    }
}
