//! The plain file a draft round-trips through the user's editor.
//!
//! Headers (`To:`, `Cc:`, `Bcc:`, `Subject:`, and `Attach:` lines naming files), a blank line,
//! then the body, so any editor works.

use std::path::PathBuf;

use anyhow::{Result, bail};

use super::Draft;
use crate::system;

/// The headers of an edited file, before they replace a draft's.
#[derive(Default)]
struct EditedHeaders {
    /// The `From:` value, `None` when blank.
    from: Option<String>,

    /// Every `To:` address.
    to: Vec<String>,

    /// Every `Cc:` address.
    cc: Vec<String>,

    /// Every `Bcc:` address.
    bcc: Vec<String>,

    /// The `Subject:` value.
    subject: String,

    /// Every `Attach:` path, `~` expanded.
    attach: Vec<PathBuf>,
}

impl Draft {
    /// The draft as the file the editor opens.
    pub fn to_file(&self) -> String {
        let mut lines: Vec<String> = self
            .from
            .iter()
            .map(|from| format!("From: {from}"))
            .collect();
        lines.push(format!("To: {}", self.to.join(", ")));
        lines.push(format!("Cc: {}", self.cc.join(", ")));
        lines.push(format!("Bcc: {}", self.bcc.join(", ")));
        lines.push(format!("Subject: {}", self.subject));
        lines.extend(
            self.attach
                .iter()
                .map(|path| format!("Attach: {}", path.display())),
        );
        let body = self.body.strip_suffix('\n').unwrap_or(&self.body);
        format!("{}\n\n{body}\n", lines.join("\n"))
    }

    /// Replaces the draft's headers and body with the edited file's.
    ///
    /// # Errors
    ///
    /// Fails on a header line nitidus doesn't know or an `Attach:` path that isn't a file, leaving
    /// the draft as it was.
    pub fn update_from_file(&mut self, text: &str) -> Result<()> {
        let (head, body) = text.split_once("\n\n").unwrap_or((text, ""));
        let headers = parse_headers(head)?;
        check_attachments(&headers.attach)?;
        self.from = headers.from;
        self.to = headers.to;
        self.cc = headers.cc;
        self.bcc = headers.bcc;
        self.subject = headers.subject;
        self.attach = headers.attach;
        body.clone_into(&mut self.body);
        Ok(())
    }
}

/// The header block, one `Key: value` per line.
fn parse_headers(head: &str) -> Result<EditedHeaders> {
    let mut headers = EditedHeaders::default();
    for line in head.lines() {
        let Some((key, value)) = line.split_once(':') else {
            bail!("not a header line: {line:?}");
        };
        headers.set(&key.trim().to_ascii_lowercase(), value.trim())?;
    }
    Ok(headers)
}

impl EditedHeaders {
    /// Records one header; `key` is lowercased.
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "from" => self.from = (!value.is_empty()).then(|| value.to_owned()),
            "to" => self.to.extend(split_addresses(value)),
            "cc" => self.cc.extend(split_addresses(value)),
            "bcc" => self.bcc.extend(split_addresses(value)),
            "subject" => value.clone_into(&mut self.subject),
            "attach" if value.is_empty() => {}
            "attach" => self.attach.push(system::expand_home(value)),
            other => bail!("unknown header {other:?}"),
        }
        Ok(())
    }
}

/// Fails naming the first path that isn't a file.
fn check_attachments(paths: &[PathBuf]) -> Result<()> {
    match paths.iter().find(|path| !path.is_file()) {
        Some(path) => bail!("attachment not found: {}", path.display()),
        None => Ok(()),
    }
}

/// Splits an address list on commas outside quotes and angle brackets.
pub fn split_addresses(list: &str) -> Vec<String> {
    let mut addresses = Vec::new();
    let mut current = String::new();
    let (mut quoted, mut angled) = (false, false);
    for c in list.chars() {
        match c {
            '"' => quoted = !quoted,
            '<' if !quoted => angled = true,
            '>' if !quoted => angled = false,
            ',' if !quoted && !angled => {
                addresses.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    addresses.push(current);
    addresses
        .iter()
        .map(|a| a.trim().to_owned())
        .filter(|a| !a.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::{ComposeKind, Template};
    use super::*;

    #[test]
    fn addresses_split_outside_quotes() {
        let list = split_addresses(r#""Hopper, Grace" <g@x.test>, ada@x.test,"#);

        assert_eq!(list, [r#""Hopper, Grace" <g@x.test>"#, "ada@x.test"]);
    }

    #[test]
    fn a_draft_round_trips_through_its_file() {
        let mut draft = Draft::from_template(ComposeKind::New, Template::default());
        draft.to = vec!["a@x.test".into()];
        draft.subject = "Hi".into();
        draft.body = "Body\n\nmore\n".into();
        let mut edited = draft.clone();

        edited
            .update_from_file(&draft.to_file())
            .expect("a valid file");

        assert_eq!(
            (edited.to, edited.subject, edited.body),
            (draft.to, draft.subject, draft.body)
        );
    }

    #[test]
    fn an_unknown_header_is_refused() {
        let mut draft = Draft::from_template(ComposeKind::New, Template::default());

        let error = draft
            .update_from_file("X-Bogus: 1\n\nbody")
            .expect_err("a refusal");

        assert_eq!(error.to_string(), r#"unknown header "x-bogus""#);
    }
}
