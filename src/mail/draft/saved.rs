//! Reading a saved draft back as a template, for editing it again.

use super::Template;
use super::file::split_addresses;
use crate::mail::envelope::Address;

impl Template {
    /// The headers and text of a saved draft's raw source.
    pub fn from_saved(raw: &[u8]) -> Self {
        let text = String::from_utf8_lossy(raw).replace("\r\n", "\n");
        let (head, body) = text.split_once("\n\n").unwrap_or((&text, ""));
        let mut template = Self {
            body: body.to_owned(),
            ..Self::default()
        };
        for (key, value) in head.lines().filter_map(|line| line.split_once(':')) {
            template.set(&key.to_ascii_lowercase(), value.trim());
        }
        template
    }

    /// Records one header of the saved draft; others are ignored.
    fn set(&mut self, key: &str, value: &str) {
        match key {
            "from" => self.from = Some(bare(value.to_owned())),
            "to" => self.to = addresses(value),
            "cc" => self.cc = addresses(value),
            "bcc" => self.bcc = addresses(value),
            "subject" => self.subject = Some(value.to_owned()),
            _ => {}
        }
    }
}

/// An address list, each kept as written.
fn addresses(value: &str) -> Vec<Address> {
    split_addresses(value).into_iter().map(bare).collect()
}

/// An address with no separate display name.
fn bare(email: String) -> Address {
    Address { name: None, email }
}
