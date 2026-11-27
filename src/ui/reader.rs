//! The message reader: a tinted header card above the body, or the raw source.

mod body;
mod source;

use itertools::Itertools as _;
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use self::body::{body_lines, reflow};
use super::card::{Card, Field, Tone};
use super::layout::{Placeholder, at_most_wide, below, cells};
use super::movement::{Movement, Navigate};
use super::{dates, theme};
use crate::mail::{Address, Headers, Message};

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
    let headers = &message.headers;
    let subject = headers
        .subject
        .clone()
        .unwrap_or_else(|| "(no subject)".into());
    let title = Line::styled(
        subject,
        Style::new().fg(theme().fg).add_modifier(Modifier::BOLD),
    );
    let chips = message.attachments().map(str::to_owned).collect();
    Card {
        title,
        fields: header_fields(headers),
        chips,
    }
}

/// The sender and recipients, then the date when the message has one.
fn header_fields(headers: &Headers) -> Vec<Field> {
    let mut fields = vec![
        field("From", &headers.from, Tone::Accent),
        field("To", &headers.to, Tone::Plain),
    ];
    if !headers.cc.is_empty() {
        fields.push(field("Cc", &headers.cc, Tone::Plain));
    }
    if let Some(date) = headers.date.as_deref() {
        fields.push(Field {
            label: "Date",
            value: dates::long(date),
            tone: Tone::Plain,
        });
    }
    fields
}

/// A field listing addresses.
fn field(label: &'static str, addresses: &[Address], tone: Tone) -> Field {
    let value = addresses.iter().join(", ");
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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::assert_snapshot;

    use super::*;
    use crate::mail::fixtures::message;
    use crate::ui::test_support::render;

    fn reader(text: &str) -> Reader {
        let mut reader = Reader::default();
        reader.show(message("1", "Notes on the loop", text));
        reader
    }

    #[test]
    fn a_message_shows_its_card_over_a_styled_body() {
        let mut reader = reader(indoc! {"
            Hi Rae,

            > Do we need a new card type?

            No.

            -- 
            Ada"});

        let screen = render(50, 16, |area, buf| reader.render(area, buf));

        assert_snapshot!(screen, @"

        Notes on the loop

         From  Ada Lovelace <ada@x.test>
           To  Rae Field <rae@x.test>
                ⎘ notes.pdf


        Hi Rae,

        ┃ Do we need a new card type?

        No.

        --
        Ada
        ");
    }

    #[test]
    fn the_source_view_waits_for_the_source_then_shows_it() {
        let mut reader = reader("body");
        let needs_reading = reader.toggle_source();
        let waiting = render(40, 4, |area, buf| reader.render(area, buf));
        reader.set_source(
            indoc! {"
            From: ada@x.test
            Subject: Notes

            body"}
            .into(),
        );

        let shown = render(40, 8, |area, buf| reader.render(area, buf));

        assert!(needs_reading);
        assert_snapshot!(format!("{waiting}\n---\n{shown}"), @"

           Message source


        ---

           Message source

           From: ada@x.test
           Subject: Notes

           body
        ");
    }

    #[test]
    fn scrolling_keeps_half_the_body_in_view() {
        let long = (1..=40).map(|n| format!("line {n}")).join("\n");
        let mut reader = reader(&long);
        reader.navigate(Movement::Bottom);

        let screen = render(30, 16, |area, buf| reader.render(area, buf));

        assert!(screen.contains("line 40"), "{screen}");
    }

    #[test]
    fn with_nothing_read_it_invites_a_choice() {
        let mut reader = Reader::default();

        let screen = render(30, 6, |area, buf| reader.render(area, buf));

        assert_snapshot!(screen, @"


        Select a message
        ");
    }
}
