//! What a write answers: whether it ran or queued, and which items failed.

use std::fmt;

use serde::Deserialize;

/// The answer of a write acting on messages, or of a send or save.
#[derive(Debug, Default, Deserialize)]
pub struct Receipt {
    /// Whether a sync engine applies the write later (pimdir) rather than now.
    #[serde(default)]
    pub queued: bool,

    /// What `message delete` did, `None` for other writes.
    #[serde(default)]
    pub action: Option<DeleteAction>,

    /// The items the write could not act on.
    #[serde(default)]
    pub errors: Vec<ItemError>,
}

/// What `message delete` did to the messages it was given.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DeleteAction {
    /// They moved to the trash.
    MovedToTrash,

    /// They were in the trash and are gone for good.
    Deleted,

    /// They were in the trash and were flagged `\Deleted`, an expunge still owing.
    Flagged,
}

/// One item a write could not act on.
#[derive(Clone, Debug, Deserialize)]
pub struct ItemError {
    /// The item's id, `None` when the backend didn't say.
    pub id: Option<String>,

    /// What went wrong.
    pub error: String,
}

impl fmt::Display for ItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "{id}: {}", self.error),
            None => f.write_str(&self.error),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_delete_receipt_says_what_happened_and_what_failed() {
        let json = json!({ "action": "moved-to-trash", "queued": true, "errors": [{ "id": "7", "error": "gone" }, { "error": "late" }] });

        let receipt: Receipt = serde_json::from_value(json).expect("a receipt");

        let errors: Vec<String> = receipt.errors.iter().map(ToString::to_string).collect();
        assert_eq!(
            (receipt.action, receipt.queued, errors),
            (
                Some(DeleteAction::MovedToTrash),
                true,
                vec!["7: gone".to_owned(), "late".to_owned()]
            )
        );
    }

    #[test]
    fn a_bare_receipt_has_nothing_to_report() {
        let receipt: Receipt = serde_json::from_value(json!({})).expect("a receipt");

        assert_eq!(
            (receipt.action, receipt.queued, receipt.errors.len()),
            (None, false, 0)
        );
    }
}
