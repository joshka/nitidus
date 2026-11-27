//! The key reference overlay, drawn from the keymap's own help table.

use ratatui::prelude::*;
use ratatui::widgets::{Clear, Widget};

use super::layout::{cells, line};
use super::theme;

/// The panel's width.
const WIDTH: u16 = 48;

/// The width of the key column.
const KEY_WIDTH: usize = 8;

/// One help section: its title and each line's keys and description.
pub struct HelpSection {
    /// The section's title.
    pub title: &'static str,

    /// Each line: the keys, as typed, and what they do.
    pub lines: Vec<(String, &'static str)>,
}

/// The help overlay.
pub struct Help<'a>(pub &'a [HelpSection]);

impl Help<'_> {
    /// The lines the sections take: a title, the lines and a gap each, plus padding.
    fn height(&self) -> u16 {
        let lines: usize = self.0.iter().map(|section| section.lines.len() + 2).sum();
        cells(lines + 2)
    }
}

impl Widget for Help<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let panel = self.panel(area);
        Clear.render(panel, buf);
        buf.set_style(panel, t.base(t.header));
        let inner = panel.inner(Margin::new(3, 1));
        let mut row = 0;
        for section in self.0 {
            render_section(section, inner, row, buf);
            row += cells(section.lines.len()) + 2;
        }
    }
}

impl Help<'_> {
    /// The panel, centred in `area`.
    fn panel(&self, area: Rect) -> Rect {
        let width = WIDTH.min(area.width);
        let height = self.height().min(area.height);
        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

/// A section's title at `row`, then its lines.
fn render_section(section: &HelpSection, area: Rect, row: u16, buf: &mut Buffer) {
    let title = Style::new().fg(theme().accent).add_modifier(Modifier::BOLD);
    Line::styled(section.title, title).render(line(area, row), buf);
    for (offset, (keys, text)) in (1..).zip(&section.lines) {
        key_line(keys, text).render(line(area, row + offset), buf);
    }
}

/// One line: the keys right-aligned in a bold column, then what they do.
fn key_line(keys: &str, text: &str) -> Line<'static> {
    let t = theme();
    Line::from(vec![
        Span::styled(
            format!("{keys:>KEY_WIDTH$}"),
            Style::new().fg(t.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("   {text}"), Style::new().fg(t.dim)),
    ])
}
