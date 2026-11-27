//! The floating panel taking keys, what its answer is for, and acting on that answer.

use ratatui::crossterm::event::KeyEvent;

use super::App;
use crate::mail::Selection;
use crate::ui::overlay::{Confirm, Outcome, Picker, Prompt};

/// A floating panel.
pub enum Overlay {
    /// The key reference; any key closes it.
    Help,

    /// A list to pick from.
    Picker(Picker<String>, PickFor),

    /// A line of text to type.
    Prompt(Prompt, PromptFor),

    /// A yes/no question.
    Confirm(Confirm, ConfirmFor),
}

/// What a picked item is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickFor {
    /// The mailbox to move the selection to.
    MoveTo,

    /// The mailbox to copy the selection to.
    CopyTo,

    /// The account to switch to.
    Account,

    /// A link to open.
    Link,

    /// A downloaded attachment to open.
    File,
}

/// What typed text is for.
pub enum PromptFor {
    /// A search of the open mailbox.
    Search,

    /// The name of a mailbox to create.
    CreateMailbox,

    /// The new name of the mailbox with this id.
    RenameMailbox(String),

    /// A file to attach to the draft.
    Attach,
}

/// What a yes is for.
pub enum ConfirmFor {
    /// Deleting these messages for good, from the trash.
    DeleteForever(Selection),

    /// Deleting the mailbox with this id.
    DeleteMailbox(String),

    /// Throwing the draft away.
    Discard,
}

impl App {
    /// Gives `key` to the open panel, which stays open, closes, or answers.
    pub(super) fn on_overlay_key(&mut self, overlay: Overlay, key: KeyEvent) {
        match overlay {
            Overlay::Help => {}
            Overlay::Picker(picker, purpose) => self.on_picker_key(picker, purpose, key),
            Overlay::Prompt(prompt, purpose) => self.on_prompt_key(prompt, purpose, key),
            Overlay::Confirm(confirm, purpose) => self.on_confirm_key(confirm, purpose, key),
        }
    }

    /// A key for the picker.
    fn on_picker_key(&mut self, mut picker: Picker<String>, purpose: PickFor, key: KeyEvent) {
        match picker.handle(key) {
            Outcome::Open => self.overlay = Some(Overlay::Picker(picker, purpose)),
            Outcome::Cancelled => {}
            Outcome::Done(choice) => self.picked(purpose, choice),
        }
    }

    /// A key for the prompt.
    fn on_prompt_key(&mut self, mut prompt: Prompt, purpose: PromptFor, key: KeyEvent) {
        match prompt.handle(key) {
            Outcome::Open => self.overlay = Some(Overlay::Prompt(prompt, purpose)),
            Outcome::Cancelled => {}
            Outcome::Done(text) => self.prompted(purpose, text),
        }
    }

    /// A key for the question.
    fn on_confirm_key(&mut self, confirm: Confirm, purpose: ConfirmFor, key: KeyEvent) {
        match Confirm::handle(key) {
            Outcome::Open => self.overlay = Some(Overlay::Confirm(confirm, purpose)),
            Outcome::Cancelled => {}
            Outcome::Done(()) => self.confirmed(purpose),
        }
    }

    /// Acts on a picked item.
    fn picked(&mut self, purpose: PickFor, choice: String) {
        match purpose {
            PickFor::MoveTo => self.move_to(choice),
            PickFor::CopyTo => self.copy_to(choice),
            PickFor::Account => self.switch_account(choice),
            PickFor::Link | PickFor::File => self.open_target(&choice),
        }
    }

    /// Acts on typed text.
    fn prompted(&mut self, purpose: PromptFor, text: String) {
        match purpose {
            PromptFor::Search => self.search_for(text),
            PromptFor::CreateMailbox => self.create_mailbox(text),
            PromptFor::RenameMailbox(mailbox) => self.rename_mailbox(mailbox, text),
            PromptFor::Attach => self.attach(&text),
        }
    }

    /// Acts on a yes.
    fn confirmed(&mut self, purpose: ConfirmFor) {
        match purpose {
            ConfirmFor::DeleteForever(selection) => self.delete(selection),
            ConfirmFor::DeleteMailbox(mailbox) => self.delete_mailbox(mailbox),
            ConfirmFor::Discard => self.discard_draft(),
        }
    }
}
