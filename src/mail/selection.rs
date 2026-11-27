//! Which messages a command acts on, named the way Himalaya's commands take them.

/// One message: the mailbox holding it and the id the backend knows it by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageRef {
    /// The mailbox id, as `mailbox list` gives it.
    pub mailbox: String,

    /// The message id within that mailbox (an IMAP UID, a Maildir file id).
    pub id: String,
}

/// Several messages of one mailbox, for a command that takes many ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// The mailbox id holding every message.
    pub mailbox: String,

    /// The message ids, in the order they were picked.
    pub ids: Vec<String>,
}

/// A selection going to another mailbox, by a move or a copy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    /// The messages and the mailbox they leave.
    pub from: Selection,

    /// The mailbox id they go to.
    pub to: String,
}
