//! The tinted header card above a message or a draft: a title, labeled fields, attachment chips.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use super::layout::{cells, line};
use super::theme;

/// Lines of padding above and below the card's content.
const PADDING: u16 = 1;

/// Columns of padding left and right of the card's content.
const INSET: u16 = 3;

/// The width of the field label column, right-aligned.
const LABEL_WIDTH: usize = 5;

/// A header card.
pub struct Card<'a> {
    /// The first line, usually the subject.
    pub title: Line<'a>,

    /// The labeled fields below the title.
    pub fields: Vec<Field>,

    /// Attachment names, shown as chips under the fields.
    pub chips: Vec<String>,
}

/// One labeled field of a card.
pub struct Field {
    /// The label (`From`, `To`).
    pub label: &'static str,

    /// The value.
    pub value: String,

    /// How the value is colored.
    pub tone: Tone,
}

/// How a field's value is colored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// The field that matters most, such as the sender.
    Accent,

    /// Any other field.
    Plain,

    /// A required field left empty.
    Missing,
}

impl Card<'_> {
    /// The card's height: padding, title, a gap, the fields and the chips.
    pub fn height(&self) -> u16 {
        let chips = u16::from(!self.chips.is_empty());
        2 * PADDING + 2 + cells(self.fields.len()) + chips
    }
}

impl Widget for &Card<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::new().bg(theme().header));
        let inner = area.inner(Margin::new(INSET, PADDING));
        self.title.clone().render(line(inner, 0), buf);
        let mut row = 2;
        for field in &self.fields {
            field_line(field).render(line(inner, row), buf);
            row += 1;
        }
        if !self.chips.is_empty() {
            chips_line(&self.chips).render(line(inner, row), buf);
        }
    }
}

/// A field: its label right-aligned in a faint column, then its value.
fn field_line(field: &Field) -> Line<'static> {
    let t = theme();
    let color = match field.tone {
        Tone::Accent => t.accent,
        Tone::Plain => t.dim,
        Tone::Missing => t.error,
    };
    let label = format!("{:>LABEL_WIDTH$}  ", field.label);
    Line::from(vec![
        Span::styled(label, Style::new().fg(t.faint)),
        Span::styled(field.value.clone(), Style::new().fg(color)),
    ])
}

/// The attachment chips, aligned with the field values.
fn chips_line(chips: &[String]) -> Line<'static> {
    let t = theme();
    let indent = Span::raw(" ".repeat(LABEL_WIDTH + 2));
    let chip = |name: &String| {
        [
            Span::styled(format!(" ⎘ {name} "), Style::new().fg(t.fg).bg(t.chip)),
            Span::raw(" "),
        ]
    };
    Line::from(
        std::iter::once(indent)
            .chain(chips.iter().flat_map(chip))
            .collect::<Vec<_>>(),
    )
}
