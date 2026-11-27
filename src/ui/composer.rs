//! The review screen a draft lands on after the editor: what will be sent, as a header card over
//! the body.

use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use super::card::{Card, Field, Tone};
use super::layout::{at_most_wide, below};
use super::movement::{Movement, Navigate};
use super::theme;
use crate::mail::{ComposeKind, Draft, ReplyScope};

/// The widest the body is drawn, as in the reader.
const MEASURE: u16 = 84;

/// Lines a page key scrolls.
const PAGE: u16 = 15;

/// The composer's state: the draft under review.
pub struct Composer {
    /// The draft.
    pub draft: Draft,

    /// The first body line shown.
    scroll: u16,
}

impl Composer {
    /// A composer reviewing `draft` from its top.
    pub fn new(draft: Draft) -> Self {
        Self { draft, scroll: 0 }
    }
}

impl Navigate for Composer {
    fn navigate(&mut self, movement: Movement) -> bool {
        self.scroll = movement.scroll(self.scroll, PAGE);
        false
    }
}

impl Widget for &Composer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.reader));
        let card = card(&self.draft);
        let height = card.height().min(area.height);
        card.render(Rect { height, ..area }, buf);
        let body = at_most_wide(below(area, height + 1).inner(Margin::new(3, 0)), MEASURE);
        let lines: Vec<Line<'_>> = self.draft.body.lines().map(body_line).collect();
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0))
            .render(body, buf);
    }
}

/// The draft's card: what it is and its subject, then its addresses and attachments.
fn card(draft: &Draft) -> Card<'static> {
    let t = theme();
    let subject = if draft.subject.is_empty() {
        "(no subject)".to_owned()
    } else {
        draft.subject.clone()
    };
    let title = Line::from(vec![
        Span::styled(
            format!(" {} ", kind_label(&draft.kind)),
            Style::new()
                .fg(t.bar)
                .bg(t.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(subject, Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
    ]);
    Card {
        title,
        fields: fields(draft),
        chips: draft.attachment_names().collect(),
    }
}

/// What kind of draft it is, as its badge says.
fn kind_label(kind: &ComposeKind) -> &'static str {
    match kind {
        ComposeKind::New => "New message",
        ComposeKind::Reply {
            scope: ReplyScope::Sender,
            ..
        } => "Reply",
        ComposeKind::Reply {
            scope: ReplyScope::All,
            ..
        } => "Reply all",
        ComposeKind::Forward(_) => "Forward",
        ComposeKind::Draft(_) => "Draft",
    }
}

/// The sender and recipients; an empty `To` is shown as missing.
fn fields(draft: &Draft) -> Vec<Field> {
    let from = draft.from.iter().map(|from| Field {
        label: "From",
        value: from.clone(),
        tone: Tone::Plain,
    });
    let copies = [("Cc", &draft.cc), ("Bcc", &draft.bcc)]
        .into_iter()
        .filter(|(_, list)| !list.is_empty());
    let copies = copies.map(|(label, list)| Field {
        label,
        value: list.join(", "),
        tone: Tone::Plain,
    });
    from.chain(std::iter::once(to_field(&draft.to)))
        .chain(copies)
        .collect()
}

/// The `To` field, shown as missing when empty, since a draft can't be sent without it.
fn to_field(to: &[String]) -> Field {
    if to.is_empty() {
        Field {
            label: "To",
            value: "(none)".into(),
            tone: Tone::Missing,
        }
    } else {
        Field {
            label: "To",
            value: to.join(", "),
            tone: Tone::Accent,
        }
    }
}

/// A body line, quotes dimmed.
fn body_line(text: &str) -> Line<'static> {
    let t = theme();
    let color = if text.trim_start().starts_with('>') {
        t.dim
    } else {
        t.fg
    };
    Line::styled(text.to_owned(), Style::new().fg(color))
}
