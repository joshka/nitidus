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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::mail::fixtures::mailbox;

    #[rstest]
    #[case::stated(mailbox("Spam", Some("junk")), Role::Junk)]
    #[case::by_name(mailbox("Archive", None), Role::Archive)]
    #[case::by_name_any_case(mailbox("trash", None), Role::Trash)]
    #[case::pimdir_prefixed(mailbox("imap/Archive", None), Role::Archive)]
    #[case::none(mailbox("Projects", None), Role::Other)]
    fn a_role_is_stated_or_read_from_the_name(#[case] mailbox: Mailbox, #[case] role: Role) {
        assert_eq!(mailbox.role(), role);
    }

    #[test]
    fn the_label_leaves_out_the_source_prefix() {
        let mailbox = mailbox("imap/Archive", None);

        assert_eq!(
            (
                mailbox.label(),
                mailbox.is_named("archive"),
                mailbox.is_named("imap/Archive")
            ),
            ("Archive", true, true)
        );
    }

    #[test]
    fn roles_order_the_sidebar() {
        let mut roles = vec![
            Role::Trash,
            Role::Other,
            Role::Inbox,
            Role::Sent,
            Role::Junk,
            Role::Drafts,
            Role::Archive,
        ];

        roles.sort();

        assert_eq!(
            roles,
            [
                Role::Inbox,
                Role::Drafts,
                Role::Sent,
                Role::Archive,
                Role::Other,
                Role::Junk,
                Role::Trash
            ]
        );
    }
}
