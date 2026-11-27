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
    /// The sections split into columns: one when they fit `height` rows, else two of about equal
    /// length, so a short terminal still shows every key.
    fn columns(&self, height: u16) -> Vec<&[HelpSection]> {
        let total: usize = self.0.iter().map(section_rows).sum();
        if cells(total + 2) <= height {
            return vec![self.0];
        }
        let mut rows = 0;
        let split = self.0.iter().position(|section| {
            rows += section_rows(section);
            rows * 2 >= total
        });
        let (left, right) = self
            .0
            .split_at(split.map_or(self.0.len(), |index| index + 1));
        vec![left, right]
    }

    /// The panel for `columns`, centred in `area`.
    fn panel(columns: &[&[HelpSection]], area: Rect) -> Rect {
        let tallest = columns
            .iter()
            .map(|column| column.iter().map(section_rows).sum::<usize>())
            .max()
            .unwrap_or_default();
        let width = (WIDTH * cells(columns.len())).min(area.width);
        let height = cells(tallest + 2).min(area.height);
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

impl Widget for Help<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let columns = self.columns(area.height);
        let panel = Help::panel(&columns, area);
        Clear.render(panel, buf);
        buf.set_style(panel, t.base(t.header));
        let inner = panel.inner(Margin::new(3, 1));
        let constraints = vec![Constraint::Fill(1); columns.len()];
        let areas = Layout::horizontal(constraints).split(inner);
        for (column, &column_area) in columns.iter().zip(areas.iter()) {
            render_column(column, column_area, buf);
        }
    }
}

/// The rows a section takes: its title, its lines and a gap.
fn section_rows(section: &HelpSection) -> usize {
    section.lines.len() + 2
}

/// Sections one under another.
fn render_column(sections: &[HelpSection], area: Rect, buf: &mut Buffer) {
    let mut row = 0;
    for section in sections {
        render_section(section, area, row, buf);
        row += cells(section_rows(section));
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

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::ui::test_support::render;

    #[test]
    fn a_short_screen_gets_two_columns_so_every_key_shows() {
        let sections = [
            HelpSection {
                title: "Move",
                lines: vec![("j k".into(), "next / previous"), ("enter".into(), "open")],
            },
            HelpSection {
                title: "Write",
                lines: vec![("c".into(), "compose")],
            },
        ];

        let screen = render(100, 6, |area, buf| Help(&sections).render(area, buf));

        assert_snapshot!(screen, @"

        Move                                         Write
             j k   next / previous                          c   compose
           enter   open
        ");
    }

    #[test]
    fn sections_list_keys_beside_what_they_do() {
        let sections = [
            HelpSection {
                title: "Move",
                lines: vec![("j k".into(), "next / previous"), ("enter".into(), "open")],
            },
            HelpSection {
                title: "Write",
                lines: vec![("c".into(), "compose")],
            },
        ];

        let screen = render(40, 10, |area, buf| Help(&sections).render(area, buf));

        assert_snapshot!(screen, @"

        Move
             j k   next / previous
           enter   open

        Write
               c   compose
        ");
    }
}
