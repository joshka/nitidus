//! Triage on the marked messages, else the selected one: flags, archive, delete, move and copy.
//! The list changes at once; the worker confirms, and a failure reloads the list.

use super::App;
use super::action::Triage;
use super::overlay::{ConfirmFor, Overlay, PickFor};
use crate::mail::{FlagChange, Keyword, Role, Selection, Transfer};
use crate::ui::overlay::{Confirm, Picker};
use crate::worker::{FlagUpdate, Request};

impl App {
    /// Performs a triage action.
    pub(super) fn triage(&mut self, action: Triage) {
        match action {
            Triage::ToggleSeen => self.toggle_keyword(Keyword::Seen),
            Triage::ToggleFlag => self.toggle_keyword(Keyword::Flagged),
            Triage::Archive => self.archive(),
            Triage::Delete => self.delete_or_confirm(),
            Triage::MoveTo => self.pick_destination(PickFor::MoveTo),
            Triage::CopyTo => self.pick_destination(PickFor::CopyTo),
        }
    }

    /// The messages triage acts on, `None` when the list is empty.
    fn selection(&self) -> Option<Selection> {
        let ids = self.list.targets();
        (!ids.is_empty()).then(|| Selection {
            mailbox: self.mailbox(),
            ids,
        })
    }

    /// Sets `keyword` on the selection, or clears it when every message already has it.
    fn toggle_keyword(&mut self, keyword: Keyword) {
        let Some(selection) = self.selection() else {
            return;
        };
        let change = if self.list.all_have(&selection.ids, keyword) {
            FlagChange::Remove
        } else {
            FlagChange::Add
        };
        self.list
            .edit(&selection.ids, |envelope| envelope.apply(keyword, change));
        self.request(
            "Updating",
            Request::Flag(FlagUpdate {
                selection,
                keyword,
                change,
            }),
        );
    }

    /// Moves the selection to the archive mailbox.
    fn archive(&mut self) {
        match self
            .sidebar
            .find(Role::Archive)
            .map(|mailbox| mailbox.id.clone())
        {
            None => self.error("No archive mailbox"),
            Some(archive) if archive == self.mailbox() => {}
            Some(archive) => self.move_to(archive),
        }
    }

    /// Deletes the selection, asking first inside the trash, where it is for good.
    fn delete_or_confirm(&mut self) {
        let Some(selection) = self.selection() else {
            return;
        };
        if !self.in_role(Role::Trash) {
            self.delete(selection);
            return;
        }
        let count = selection.ids.len();
        let question = format!("Delete {count} message{} for good?", plural(count));
        self.overlay = Some(Overlay::Confirm(
            Confirm::new(question),
            ConfirmFor::DeleteForever(selection),
        ));
    }

    /// Deletes `selection`, dropping it from the list at once.
    pub(super) fn delete(&mut self, selection: Selection) {
        self.drop_from_view(&selection.ids);
        self.request("Deleting", Request::Delete(selection));
    }

    /// Asks which mailbox the selection goes to.
    fn pick_destination(&mut self, purpose: PickFor) {
        let Some(selection) = self.selection() else {
            return;
        };
        let verb = if purpose == PickFor::MoveTo {
            "Move"
        } else {
            "Copy"
        };
        let count = selection.ids.len();
        let title = format!("{verb} {count} message{} to…", plural(count));
        let others = self
            .sidebar
            .mailboxes()
            .iter()
            .filter(|mailbox| mailbox.id != selection.mailbox);
        let items = others
            .map(|mailbox| (mailbox.label().to_owned(), mailbox.id.clone()))
            .collect();
        self.overlay = Some(Overlay::Picker(Picker::new(title, items), purpose));
    }

    /// Moves the selection to the mailbox with id `to`.
    pub(super) fn move_to(&mut self, to: String) {
        let Some(from) = self.selection() else { return };
        self.drop_from_view(&from.ids);
        self.request("Moving", Request::Move(Transfer { from, to }));
    }

    /// Copies the selection to the mailbox with id `to`.
    pub(super) fn copy_to(&mut self, to: String) {
        let Some(from) = self.selection() else { return };
        self.list.clear_marks();
        self.request("Copying", Request::Copy(Transfer { from, to }));
    }

    /// Takes messages out of the list and reads whatever is selected next.
    fn drop_from_view(&mut self, ids: &[String]) {
        self.list.clear_marks();
        self.list.remove(ids);
        self.reader.clear();
        self.open_current();
    }
}

/// The plural suffix for `count` messages.
pub(super) fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
