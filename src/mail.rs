//! The mail side: Himalaya's own commands, run in-process over one session.
//!
//! nitidus couples to Himalaya's versioned `--json` output, never its Rust types: each operation
//! builds an argv, Himalaya's clap grammar parses it, and the printed JSON is decoded into the
//! small types of this module. See `docs/conventions/mail.md`.

mod client;
mod draft;
mod envelope;
mod mailbox;
mod message;
mod receipt;
mod selection;

#[cfg(test)]
pub mod fixtures;
#[cfg(test)]
mod pimdir_tests;
#[cfg(test)]
mod tests;

pub use client::MailClient;
pub use draft::{ComposeKind, Draft, ReplyScope, Submit, Template};
pub use envelope::{Address, Envelope, EnvelopePage, FlagChange, Keyword, PageQuery};
pub use mailbox::{Mailbox, Role};
pub use message::{Headers, Message};
pub use receipt::{DeleteAction, Receipt};
pub use selection::{MessageRef, Selection, Transfer};
