//! A message as `message read` prints it: decoded headers, text, HTML and parts.

use serde::Deserialize;

use super::envelope::Address;

/// The narrowest width HTML is rendered at, so a tiny pane still gets readable text.
const MIN_HTML_WIDTH: usize = 20;

/// The schemes the link picker offers.
const LINK_SCHEMES: [&str; 3] = ["https://", "http://", "mailto:"];

/// A message, read.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// The id the message was read by.
    pub id: String,

    /// The headers a reader acts on, decoded.
    pub headers: Headers,

    /// Every `text/plain` part joined, `None` when there is none.
    pub text: Option<String>,

    /// The first `text/html` part, `None` when there is none.
    pub html: Option<String>,

    /// Every leaf part, depth first.
    #[serde(default)]
    parts: Vec<Part>,
}

/// The decoded headers of a message.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Headers {
    /// The senders.
    #[serde(default)]
    pub from: Vec<Address>,

    /// The primary recipients.
    #[serde(default)]
    pub to: Vec<Address>,

    /// The carbon-copy recipients.
    #[serde(default)]
    pub cc: Vec<Address>,

    /// The decoded subject.
    pub subject: Option<String>,

    /// The `Date:` header as written.
    pub date: Option<String>,
}

/// One leaf part; only attachments carry a file name.
#[derive(Clone, Debug, Deserialize)]
struct Part {
    /// The decoded file name, `None` for an inline body part.
    #[serde(default)]
    filename: Option<String>,
}

impl Message {
    /// The body as plain text: the text part, else the HTML rendered to `width` columns.
    pub fn body(&self, width: usize) -> String {
        if let Some(text) = &self.text {
            return text.replace("\r\n", "\n");
        }
        self.html
            .as_deref()
            .map_or_default(|html| render_html(html, width))
    }

    /// The file names of the attachments, in order.
    pub fn attachments(&self) -> impl Iterator<Item = &str> {
        self.parts
            .iter()
            .filter_map(|part| part.filename.as_deref())
    }

    /// Every http(s) and mailto link in the text and HTML parts, in order, without repeats.
    pub fn links(&self) -> Vec<String> {
        let text = self.text.as_deref().unwrap_or_default();
        let html = self.html.as_deref().unwrap_or_default();
        let mut links = Vec::new();
        for link in LINK_SCHEMES
            .iter()
            .flat_map(|scheme| find_links([text, html], scheme))
        {
            if !links.contains(&link) {
                links.push(link);
            }
        }
        links
    }
}

/// `html` as plain text wrapped at `width`, the HTML itself when it won't parse.
fn render_html(html: &str, width: usize) -> String {
    let rendered = html2text::from_read(html.as_bytes(), width.max(MIN_HTML_WIDTH));
    rendered.unwrap_or_else(|_| html.to_owned())
}

/// The links starting with `scheme` in each of `texts`.
fn find_links<'a>(texts: [&'a str; 2], scheme: &'a str) -> impl Iterator<Item = String> + 'a {
    texts.into_iter().flat_map(move |text| {
        text.match_indices(scheme)
            .map(|(at, _)| link_at(&text[at..]))
    })
}

/// The link at the start of `tail`, up to whitespace or a closing delimiter, minus trailing
/// punctuation.
fn link_at(tail: &str) -> String {
    let end = tail.find(ends_link).unwrap_or(tail.len());
    tail[..end].trim_end_matches(['.', ',', ';']).to_owned()
}

/// Whether `c` ends a link written in prose or in an HTML attribute.
fn ends_link(c: char) -> bool {
    c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ']')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(text: &str, html: &str) -> Message {
        let json = serde_json::json!({ "id": "1", "headers": {}, "text": text, "html": html });
        serde_json::from_value(json).expect("a message")
    }

    #[test]
    fn the_text_part_wins_with_line_endings_made_plain() {
        let message = message("one\r\ntwo", "<p>ignored</p>");

        assert_eq!(message.body(80), "one\ntwo");
    }

    #[test]
    fn html_alone_is_rendered_as_text() {
        let json = serde_json::json!({ "id": "1", "headers": {}, "html": "<h1>Title</h1><p>Body text</p>" });
        let message: Message = serde_json::from_value(json).expect("a message");

        let body = message.body(40);

        assert!(
            body.contains("Title") && body.contains("Body text") && !body.contains("<p>"),
            "{body}"
        );
    }

    #[test]
    fn attachments_are_the_named_parts() {
        let json = serde_json::json!({ "id": "1", "headers": {}, "parts": [{ "filename": null }, { "filename": "a.pdf" }] });
        let message: Message = serde_json::from_value(json).expect("a message");

        assert_eq!(message.attachments().collect::<Vec<_>>(), ["a.pdf"]);
    }

    #[test]
    fn links_come_from_text_and_html_once_each() {
        let m = message(
            "See https://ratatui.rs. Or mail mailto:ada@x.test, please.",
            r#"<a href="https://ratatui.rs">ratatui</a>"#,
        );

        assert_eq!(m.links(), ["https://ratatui.rs", "mailto:ada@x.test"]);
    }
}
