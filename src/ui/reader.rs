//! The message reader: a tinted header card above the body, or the raw source.

mod body;
mod source;

use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use self::body::{body_lines, reflow};
use super::card::{Card, Field, Tone};
use super::layout::{Placeholder, at_most_wide, below, cells};
use super::movement::{Movement, Navigate};
use super::{dates, theme};
use crate::mail::{Address, Message};

/// The widest the body is drawn: mail is mostly hard-wrapped near 72–78 columns.
const MEASURE: u16 = 84;

/// Lines a page key scrolls.
const PAGE: u16 = 15;

/// The reader's state.
#[derive(Default)]
pub struct Reader {
    /// The message shown, `None` before one is read.
    message: Option<Message>,

    /// The message's raw source, once read.
    source: Option<String>,

    /// Which view is shown.
    view: View,

    /// Whether a message is being read.
    pub loading: bool,

    /// The first body line shown.
    scroll: u16,
}

/// What the reader shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    /// The decoded message.
    #[default]
    Message,

    /// The raw source.
    Source,
}

impl Reader {
    /// The message shown.
    pub fn message(&self) -> Option<&Message> {
        self.message.as_ref()
    }

    /// Whether the message with `id` is shown.
    pub fn shows(&self, id: &str) -> bool {
        self.message
            .as_ref()
            .is_some_and(|message| message.id == id)
    }

    /// Clears the reader, ready for the next message.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Shows a message just read.
    pub fn show(&mut self, message: Message) {
        self.message = Some(message);
        self.loading = false;
    }

    /// Keeps a source just read, for the source view.
    pub fn set_source(&mut self, source: String) {
        self.source = Some(source);
    }

    /// Switches between the message and its source, answering whether the source still needs
    /// reading.
    pub fn toggle_source(&mut self) -> bool {
        self.view = match self.view {
            View::Message => View::Source,
            View::Source => View::Message,
        };
        self.scroll = 0;
        self.view == View::Source && self.source.is_none()
    }

    /// Whether the source view is shown.
    pub fn showing_source(&self) -> bool {
        self.view == View::Source
    }
}

impl Navigate for Reader {
    fn navigate(&mut self, movement: Movement) -> bool {
        self.scroll = movement.scroll(self.scroll, PAGE);
        false
    }
}

impl Widget for &mut Reader {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let t = theme();
        buf.set_style(area, t.base(t.reader));
        match (self.view, &self.message) {
            (View::Source, _) => {
                source::render(self.source.as_deref(), &mut self.scroll, area, buf);
            }
            (View::Message, Some(message)) => render_message(message, &mut self.scroll, area, buf),
            (View::Message, None) => Placeholder(if self.loading {
                "Loading…"
            } else {
                "Select a message"
            })
            .render(area, buf),
        }
    }
}

/// The header card, then the body below it.
fn render_message(message: &Message, scroll: &mut u16, area: Rect, buf: &mut Buffer) {
    let card = card(message);
    let height = card.height().min(area.height);
    card.render(Rect { height, ..area }, buf);
    let body = at_most_wide(below(area, height + 1).inner(Margin::new(3, 0)), MEASURE);
    let lines = body_lines(&reflow(&message.body(usize::from(body.width))));
    render_scrolled(lines, scroll, body, buf);
}

/// The message's header card: subject, sender, recipients, date and attachments.
fn card(message: &Message) -> Card<'static> {
    let t = theme();
    let headers = &message.headers;
    let subject = headers
        .subject
        .clone()
        .unwrap_or_else(|| "(no subject)".into());
    let mut fields = vec![
        field("From", &headers.from, Tone::Accent),
        field("To", &headers.to, Tone::Plain),
    ];
    if !headers.cc.is_empty() {
        fields.push(field("Cc", &headers.cc, Tone::Plain));
    }
    let date = headers.date.as_deref().map_or_default(dates::long);
    fields.push(Field {
        label: "Date",
        value: date,
        tone: Tone::Plain,
    });
    let title = Line::styled(subject, Style::new().fg(t.fg).add_modifier(Modifier::BOLD));
    Card {
        title,
        fields,
        chips: message.attachments().map(str::to_owned).collect(),
    }
}

/// A field listing addresses.
fn field(label: &'static str, addresses: &[Address], tone: Tone) -> Field {
    let value = addresses
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    Field { label, value, tone }
}

/// `lines` wrapped into `area`, scrolled by `scroll`, which is kept so half a screen still shows.
fn render_scrolled(lines: Vec<Line<'static>>, scroll: &mut u16, area: Rect, buf: &mut Buffer) {
    let max = cells(lines.len()).saturating_sub(area.height / 2);
    *scroll = (*scroll).min(max);
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((*scroll, 0))
        .render(area, buf);
}
