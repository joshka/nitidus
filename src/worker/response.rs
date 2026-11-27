//! What the worker answers.

use super::request::{AfterDownload, Listing};
use crate::mail::{Draft, EnvelopePage, Mailbox, Message, MessageRef, Receipt, Submit};

/// An answer to one request.
pub enum Response {
    /// The configured account names.
    Accounts(Vec<String>),

    /// The account's mailboxes.
    Mailboxes(Vec<Mailbox>),

    /// A page, with the listing it answers.
    Envelopes(Listing, EnvelopePage),

    /// A message, read.
    Message(Box<Message>),

    /// A message's raw source.
    Source(MessageRef, String),

    /// A write on messages finished.
    Wrote(Write, Receipt),

    /// A mailbox was created, renamed or deleted; the text says which.
    MailboxesChanged(String),

    /// A draft, ready for the editor.
    Template(Box<Draft>),

    /// A draft was sent or saved.
    Submitted(Submit, Receipt),

    /// Attachments were written to these paths.
    Downloaded(Vec<String>, AfterDownload),

    /// The request failed.
    Failed(Failure),
}

/// A write on messages, described for the status bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Write {
    /// A keyword was set or cleared.
    Flag,

    /// Messages moved to `to`.
    Move {
        /// How many.
        count: usize,

        /// The destination mailbox.
        to: String,
    },

    /// Messages copied to `to`.
    Copy {
        /// How many.
        count: usize,

        /// The destination mailbox.
        to: String,
    },

    /// Messages deleted or moved to the trash.
    Delete {
        /// How many.
        count: usize,
    },
}

/// A request that failed.
pub struct Failure {
    /// What was being done, as the status bar names it.
    pub action: &'static str,

    /// The error with its whole chain of causes.
    pub error: String,
}

impl Response {
    /// A failure of `action`, keeping every cause of `error`.
    pub fn failed(action: &'static str, error: &anyhow::Error) -> Self {
        Self::Failed(Failure {
            action,
            error: format!("{error:#}"),
        })
    }
}
