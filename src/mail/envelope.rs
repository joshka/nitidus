//! Envelopes: the listing row of a message, its flags, and the page it came in.

use std::collections::BTreeSet;
use std::fmt;

use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

/// How many envelopes a page asks for.
pub const PAGE_SIZE: usize = 50;

/// One page of a mailbox listing or search, as `envelope list|search` prints it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopePage {
    /// The envelopes of the page, in the backend's order.
    pub envelopes: Vec<Envelope>,

    /// The cursor of the next page, `None` once the listing is complete.
    pub next_page: Option<String>,
}

/// What to list: a mailbox, an optional search over it, and which page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageQuery {
    /// The mailbox id.
    pub mailbox: String,

    /// A query in the pimdir search language (`from:ada is:unread`), `None` for a plain listing.
    pub search: Option<String>,

    /// The page, the first being 1.
    pub page: usize,

    /// Envelopes per page.
    pub size: usize,
}

impl PageQuery {
    /// The first page of `mailbox`, searched by `search` when given.
    pub fn first(mailbox: String, search: Option<String>) -> Self {
        Self {
            mailbox,
            search,
            page: 1,
            size: PAGE_SIZE,
        }
    }

    /// The same listing at another page.
    pub fn at_page(&self, page: usize) -> Self {
        Self {
            page,
            ..self.clone()
        }
    }

    /// Whether `other` lists the same mailbox with the same search, whatever the page.
    pub fn same_listing(&self, other: &Self) -> bool {
        self.mailbox == other.mailbox && self.search == other.search
    }
}

/// A message's listing row.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    /// The id commands name the message by.
    pub id: String,

    /// The flags set on the message.
    #[serde(default)]
    pub flags: BTreeSet<Flag>,

    /// The decoded `Subject:`, `None` when the message has none.
    #[serde(default)]
    pub subject: Option<String>,

    /// The senders.
    #[serde(default)]
    pub from: Vec<Address>,

    /// The `Date:` header, `None` when missing or unparsable.
    #[serde(default)]
    pub date: Option<DateTime<FixedOffset>>,

    /// Whether the message carries an attachment, `None` when the backend can't tell.
    #[serde(default)]
    pub has_attachment: Option<bool>,

    /// The normalised `Message-ID:`, which replies point at.
    #[serde(default)]
    pub message_id: Option<String>,

    /// The `In-Reply-To:` ids, which thread a reply under its parent.
    #[serde(default)]
    pub in_reply_to: Vec<String>,
}

impl Envelope {
    /// Whether the message carries `keyword`.
    pub fn has(&self, keyword: Keyword) -> bool {
        self.flags
            .iter()
            .any(|flag| flag.iana.as_deref() == Some(keyword.name()))
    }

    /// Sets or clears `keyword` locally, ahead of the backend confirming it.
    pub fn apply(&mut self, keyword: Keyword, change: FlagChange) {
        self.flags
            .retain(|flag| flag.iana.as_deref() != Some(keyword.name()));
        if change == FlagChange::Add {
            self.flags.insert(Flag::from(keyword));
        }
    }

    /// The first sender's display name, or a placeholder.
    pub fn sender(&self) -> &str {
        self.from.first().map_or("(unknown)", Address::display_name)
    }

    /// The subject, or a placeholder.
    pub fn subject_or_placeholder(&self) -> &str {
        self.subject.as_deref().unwrap_or("(no subject)")
    }
}

/// A flag as the backend spells it, with the IANA keyword it stands for.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flag {
    /// The backend's spelling (`\Seen`, a Maildir letter).
    pub raw: String,

    /// The IANA keyword (`seen`, `flagged`), `None` for a custom keyword.
    pub iana: Option<String>,
}

impl From<Keyword> for Flag {
    fn from(keyword: Keyword) -> Self {
        Self {
            raw: keyword.name().into(),
            iana: Some(keyword.name().into()),
        }
    }
}

/// The keywords nitidus reads and sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyword {
    /// The message has been read.
    Seen,

    /// The message is marked for attention.
    Flagged,

    /// The message has been replied to.
    Answered,
}

impl Keyword {
    /// The name Himalaya prints and takes (`flag add -f seen`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Seen => "seen",
            Self::Flagged => "flagged",
            Self::Answered => "answered",
        }
    }
}

/// Whether a flag command sets or clears its keyword.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagChange {
    /// Set the keyword.
    Add,

    /// Clear the keyword.
    Remove,
}

impl FlagChange {
    /// The `flag` subcommand for this change.
    pub fn verb(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Remove => "remove",
        }
    }
}

/// An email address with an optional display name.
#[derive(Clone, Debug, Deserialize)]
pub struct Address {
    /// The display name, `None` when absent.
    pub name: Option<String>,

    /// The address itself.
    pub email: String,
}

impl Address {
    /// The display name when there is one, else the address.
    pub fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or(&self.email)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) if !name.is_empty() => write!(f, "{name} <{}>", self.email),
            _ => f.write_str(&self.email),
        }
    }
}
