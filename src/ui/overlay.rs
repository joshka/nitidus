//! Floating panels: a filterable picker, a one-line prompt, a yes/no question. Each owns its
//! state, handles its own keys, and draws itself centred over the screen.

mod confirm;
mod picker;
mod prompt;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Widget};

pub use self::confirm::Confirm;
pub use self::picker::Picker;
pub use self::prompt::Prompt;
use super::layout::{below, line};
use super::theme;

/// What a key did to an open panel.
pub enum Outcome<T> {
    /// The panel stays open.
    Open,

    /// The panel was dismissed.
    Cancelled,

    /// The panel closed with an answer.
    Done(T),
}

/// The keys a text field and a list understand.
enum Edit {
    /// Esc.
    Cancel,

    /// Enter.
    Submit,

    /// Backspace.
    Erase,

    /// Down, Tab, ctrl-n or ctrl-j.
    Next,

    /// Up, Shift-Tab, ctrl-p or ctrl-k.
    Previous,

    /// A printable character.
    Type(char),
}

impl Edit {
    /// The edit a key stands for, `None` for keys panels ignore.
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "panels read a handful of keys and ignore the rest"
    )]
    fn of(key: KeyEvent) -> Option<Self> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let edit = match key.code {
            KeyCode::Esc => Self::Cancel,
            KeyCode::Enter => Self::Submit,
            KeyCode::Backspace => Self::Erase,
            KeyCode::Down | KeyCode::Tab => Self::Next,
            KeyCode::Up | KeyCode::BackTab => Self::Previous,
            KeyCode::Char('n' | 'j') if ctrl => Self::Next,
            KeyCode::Char('p' | 'k') if ctrl => Self::Previous,
            KeyCode::Char(c) => Self::Type(c),
            _ => return None,
        };
        Some(edit)
    }
}

/// A panel `width` by `height`, centred across `area` and a third of the way down.
fn panel_area(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 3;
    Rect {
        x,
        y,
        width,
        height,
    }
}

/// Clears `area`, tints it, and draws `title` on a darker first row; answers the content area.
fn draw_panel(area: Rect, title: &str, buf: &mut Buffer) -> Rect {
    let t = theme();
    Clear.render(area, buf);
    buf.set_style(area, t.base(t.header));
    let title_row = line(area, 0);
    buf.set_style(title_row, Style::new().bg(t.chip));
    Line::styled(
        format!("  {title}"),
        Style::new().fg(t.fg).add_modifier(Modifier::BOLD),
    )
    .render(title_row, buf);
    below(area, 1).inner(Margin::new(2, 1))
}
