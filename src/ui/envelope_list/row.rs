//! One envelope's row, selection shown by background: sender and marks over the subject, or all on
//! one line when compact.

use ratatui::prelude::*;
use ratatui::widgets::Widget;

use crate::mail::{Envelope, Keyword};
use crate::ui::layout::{Emphasis, line};
use crate::ui::{dates, theme};

/// The deepest thread level indented; deeper replies line up with it.
const MAX_INDENT_DEPTH: u8 = 4;

/// The widest a compact row's sender column grows.
const MAX_SENDER_WIDTH: u16 = 24;

/// How one row is shown.
#[derive(Clone, Copy)]
pub struct RowState {
    /// How the selection sets it off.
    pub emphasis: Emphasis,

    /// Whether it is marked.
    pub marked: bool,

    /// Its thread depth.
    pub depth: u8,

    /// Whether it takes one line instead of two.
    pub compact: bool,
}

/// An envelope's row, drawn into `area` (two lines, or one when compact).
pub struct Row<'a> {
    /// The envelope.
    pub envelope: &'a Envelope,

    /// How it is shown.
    pub state: RowState,
}

impl Row<'_> {
    /// Whether the row is set off, by selection or mark, which brightens its text.
    fn lit(&self) -> bool {
        self.state.marked || self.state.emphasis != Emphasis::Plain
    }

    /// Whether the message is unread.
    fn unread(&self) -> bool {
        !self.envelope.has(Keyword::Seen)
    }
}

impl Widget for Row<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::new().bg(self.background()));
        self.draw_edge(area, buf);
        let text = indented(area, self.state.depth);
        let marks = Line::from(self.marks()).right_aligned();
        if self.state.compact {
            self.render_compact(line(text, 0), marks, buf);
            return;
        }
        Line::from(self.sender()).render(line(text, 0), buf);
        marks.render(line(text, 0), buf);
        self.subject().render(line(text, 1), buf);
    }
}

impl Row<'_> {
    /// Sender, subject and marks on one line: the sender in a column a third of the width (at
    /// most [`MAX_SENDER_WIDTH`]), the subject cut short before the marks.
    fn render_compact(&self, area: Rect, marks: Line<'static>, buf: &mut Buffer) {
        let marks_width = u16::try_from(marks.width()).unwrap_or(u16::MAX);
        let sender_width = (area.width / 3).min(MAX_SENDER_WIDTH);
        let [sender, subject, date] = Layout::horizontal([
            Constraint::Length(sender_width),
            Constraint::Fill(1),
            Constraint::Length(marks_width),
        ])
        .spacing(1)
        .areas(area);
        Line::from(self.sender()).render(sender, buf);
        Line::from(self.subject_span()).render(subject, buf);
        marks.render(date, buf);
    }
}

impl Row<'_> {
    /// The background: the selection's, else the dimmer one for a mark.
    fn background(&self) -> Color {
        let emphasis = match (self.state.emphasis, self.state.marked) {
            (Emphasis::Plain, true) => Emphasis::Unfocused,
            (emphasis, _) => emphasis,
        };
        emphasis.background(theme().bg)
    }

    /// The one-cell edge: green for a mark, accent for the focused selection.
    fn draw_edge(&self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        let color = match (self.state.marked, self.state.emphasis) {
            (true, _) => t.ok,
            (false, Emphasis::Focused) => t.accent,
            (false, Emphasis::Unfocused | Emphasis::Plain) => return,
        };
        for y in area.top()..area.bottom() {
            buf[(area.x, y)].set_symbol("▌").set_fg(color);
        }
    }

    /// The lead glyph and the sender, bold when unread.
    fn sender(&self) -> Vec<Span<'static>> {
        let t = theme();
        let weight = if self.unread() {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };
        let color = if self.lit() || self.unread() {
            t.fg
        } else {
            t.dim
        };
        let style = Style::new().fg(color).add_modifier(weight);
        vec![
            self.lead(),
            Span::styled(self.envelope.sender().to_owned(), style),
        ]
    }

    /// The glyph before the sender: a mark, an unread dot, a reply arrow, or nothing.
    fn lead(&self) -> Span<'static> {
        let t = theme();
        if self.state.marked {
            Span::styled("✓ ", Style::new().fg(t.ok))
        } else if self.unread() {
            Span::styled("● ", Style::new().fg(t.unread))
        } else if self.state.depth > 0 {
            Span::styled(
                "↳ ",
                Style::new().fg(if self.lit() { t.dim } else { t.faint }),
            )
        } else {
            Span::raw("  ")
        }
    }

    /// The attachment, answered and flagged glyphs, then the date.
    fn marks(&self) -> Vec<Span<'static>> {
        let t = theme();
        let envelope = self.envelope;
        let glyphs = [
            (envelope.has_attachment == Some(true), "⎘ ", t.dim),
            (envelope.has(Keyword::Answered), "↩ ", t.dim),
            (envelope.has(Keyword::Flagged), "★ ", t.flagged),
        ];
        let shown = glyphs.into_iter().filter(|&(shown, _, _)| shown);
        let mut spans: Vec<Span<'static>> = shown
            .map(|(_, glyph, color)| Span::styled(glyph, Style::new().fg(color)))
            .collect();
        let date_color = if self.lit() {
            t.fg
        } else if self.unread() {
            t.unread
        } else {
            t.faint
        };
        spans.push(Span::styled(
            dates::short(envelope.date),
            Style::new().fg(date_color),
        ));
        spans
    }

    /// The subject line, indented under the sender.
    fn subject(&self) -> Line<'static> {
        Line::from(vec![Span::raw("  "), self.subject_span()])
    }

    /// The subject, bright when unread or set off.
    fn subject_span(&self) -> Span<'static> {
        let t = theme();
        let color = if self.lit() || self.unread() {
            t.fg
        } else {
            t.dim
        };
        Span::styled(
            self.envelope.subject_or_placeholder().to_owned(),
            Style::new().fg(color),
        )
    }
}

/// The text area, inset and indented by thread depth.
fn indented(area: Rect, depth: u8) -> Rect {
    let inner = area.inner(Margin::new(2, 0));
    let indent = u16::from(depth.min(MAX_INDENT_DEPTH)) * 2;
    Rect {
        x: inner.x + indent,
        width: inner.width.saturating_sub(indent),
        ..inner
    }
}
