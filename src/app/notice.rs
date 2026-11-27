//! The status bar's transient message.

use std::time::{Duration, Instant};

use crate::ui::status::Status;

/// How long a message stays before the key hints return.
const SHOWN_FOR: Duration = Duration::from_secs(5);

/// A message for the status bar.
pub struct Notice {
    /// Whether it reports success or failure.
    kind: Kind,

    /// The text.
    text: String,

    /// When it appeared.
    shown_at: Instant,
}

/// Success or failure.
enum Kind {
    /// Something went well.
    Info,

    /// Something failed.
    Error,
}

impl Notice {
    /// A success message.
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            kind: Kind::Info,
            text: text.into(),
            shown_at: Instant::now(),
        }
    }

    /// A failure message.
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            kind: Kind::Error,
            text: text.into(),
            shown_at: Instant::now(),
        }
    }

    /// Whether it has been shown long enough.
    pub fn expired(&self) -> bool {
        self.shown_at.elapsed() > SHOWN_FOR
    }

    /// The status bar showing it.
    pub fn status(&self) -> Status<'_> {
        match self.kind {
            Kind::Info => Status::Info(&self.text),
            Kind::Error => Status::Error(&self.text),
        }
    }
}
