//! Mailboxes and the role each plays (inbox, drafts, trash…).

use serde::Deserialize;

/// A mailbox as `mailbox list` prints it.
#[derive(Clone, Debug, Deserialize)]
pub struct Mailbox {
    /// The id every other command names the mailbox by.
    pub id: String,

    /// The backend's name for it; pimdir prefixes the source (`imap/Archive`).
    pub name: String,

    /// The special-use role the backend or a `mailbox.alias` gives, if any.
    #[serde(default)]
    role: Option<String>,
}

/// What a mailbox is for, ordered as the sidebar lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// Where new mail arrives.
    Inbox,

    /// Unsent messages.
    Drafts,

    /// Copies of sent messages.
    Sent,

    /// Where archived messages go.
    Archive,

    /// A mailbox with no special role.
    Other,

    /// Junk and spam.
    Junk,

    /// Deleted messages, removed for good from here.
    Trash,
}

impl Mailbox {
    /// The name to show: the last path segment, leaving out pimdir's source prefix.
    pub fn label(&self) -> &str {
        self.name.rsplit('/').next().unwrap_or(&self.name)
    }

    /// The role the backend states, else one guessed from a conventional name.
    pub fn role(&self) -> Role {
        let stated = self.role.as_deref().and_then(Role::from_name);
        stated
            .or_else(|| Role::from_name(self.label()))
            .unwrap_or(Role::Other)
    }

    /// Whether `name` names this mailbox, by id or by label.
    pub fn is_named(&self, name: &str) -> bool {
        self.id == name || self.label().eq_ignore_ascii_case(name)
    }
}

impl Role {
    /// The role a Himalaya role name or a conventional mailbox name stands for.
    fn from_name(name: &str) -> Option<Self> {
        let role = match name.to_ascii_lowercase().as_str() {
            "inbox" => Self::Inbox,
            "drafts" => Self::Drafts,
            "sent" => Self::Sent,
            "archive" => Self::Archive,
            "junk" | "spam" => Self::Junk,
            "trash" => Self::Trash,
            _ => return None,
        };
        Some(role)
    }
}
