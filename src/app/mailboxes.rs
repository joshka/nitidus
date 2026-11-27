//! Managing mailboxes and switching accounts.

use super::App;
use super::action::MailboxAction;
use super::overlay::{ConfirmFor, Overlay, PickFor, PromptFor};
use crate::ui::envelope_list::EnvelopeList;
use crate::ui::overlay::{Confirm, Picker, Prompt};
use crate::ui::reader::Reader;
use crate::ui::sidebar::Sidebar;
use crate::worker::{AccountSpec, MailboxRename, Request, Worker};

impl App {
    /// Performs a mailbox action from the sidebar.
    pub(super) fn mailbox_action(&mut self, action: MailboxAction) {
        match action {
            MailboxAction::Create => self.prompt_new_mailbox(),
            MailboxAction::Rename => self.prompt_rename(),
            MailboxAction::Delete => self.confirm_delete_mailbox(),
        }
    }

    /// Asks for the new mailbox's name.
    fn prompt_new_mailbox(&mut self) {
        let prompt = Prompt::new("New mailbox", "", "Name of the mailbox to create");
        self.overlay = Some(Overlay::Prompt(prompt, PromptFor::CreateMailbox));
    }

    /// Asks for the open mailbox's new name.
    fn prompt_rename(&mut self) {
        let Some(mailbox) = self.sidebar.current() else {
            return;
        };
        let prompt = Prompt::new(
            format!("Rename {}", mailbox.name),
            mailbox.name.clone(),
            "New name",
        );
        self.overlay = Some(Overlay::Prompt(
            prompt,
            PromptFor::RenameMailbox(mailbox.id.clone()),
        ));
    }

    /// Asks before deleting the open mailbox.
    fn confirm_delete_mailbox(&mut self) {
        let Some(mailbox) = self.sidebar.current() else {
            return;
        };
        let confirm = Confirm::new(format!(
            "Delete mailbox {} and everything in it?",
            mailbox.name
        ));
        self.overlay = Some(Overlay::Confirm(
            confirm,
            ConfirmFor::DeleteMailbox(mailbox.id.clone()),
        ));
    }

    /// Creates a mailbox named `name`.
    pub(super) fn create_mailbox(&mut self, name: String) {
        if !name.is_empty() {
            self.request("Creating mailbox", Request::CreateMailbox(name));
        }
    }

    /// Renames the mailbox with id `mailbox` to `name`.
    pub(super) fn rename_mailbox(&mut self, mailbox: String, name: String) {
        if !name.is_empty() {
            self.request(
                "Renaming mailbox",
                Request::RenameMailbox(MailboxRename { mailbox, name }),
            );
        }
    }

    /// Deletes the mailbox with id `mailbox`.
    pub(super) fn delete_mailbox(&mut self, mailbox: String) {
        self.request("Deleting mailbox", Request::DeleteMailbox(mailbox));
    }

    /// A mailbox changed: say so and list the mailboxes again.
    pub(super) fn mailboxes_changed(&mut self, said: String) {
        self.info(said);
        self.request("Loading mailboxes", Request::Mailboxes);
    }

    /// Asks which account to switch to.
    pub(super) fn pick_account(&mut self, accounts: Vec<String>) {
        let items = accounts
            .into_iter()
            .map(|name| (name.clone(), name))
            .collect();
        self.overlay = Some(Overlay::Picker(
            Picker::new("Switch account", items),
            PickFor::Account,
        ));
    }

    /// Opens account `name` on a fresh worker, starting again from its mailboxes.
    pub(super) fn switch_account(&mut self, name: String) {
        if name == self.sidebar.account {
            return;
        }
        let spec = AccountSpec {
            config: self.config.clone(),
            account: Some(name.clone()),
        };
        self.worker = Worker::spawn(spec);
        self.pending.clear();
        self.info(format!("Switched to {name}"));
        self.sidebar = Sidebar::new(name);
        self.list = EnvelopeList::loading("Inbox".into());
        self.reader = Reader::default();
        self.search = None;
        self.set_focus(super::navigation::Focus::List);
        self.request("Loading mailboxes", Request::Mailboxes);
    }
}
