//! Performing actions: each group goes to the module that owns it.

use super::App;
use super::action::{Action, General};
use super::overlay::Overlay;
use crate::worker::Request;

impl App {
    /// Performs `action`.
    pub(super) fn perform(&mut self, action: Action) {
        match action {
            Action::General(general) => self.general(general),
            Action::Focus(movement) => self.move_focus(movement),
            Action::Move(movement) => self.move_focused(movement),
            Action::List(list) => self.list_action(list),
            Action::Triage(triage) => self.triage(triage),
            Action::Compose(compose) => self.compose(compose),
            Action::Read(read) => self.read(read),
            Action::Mailbox(mailbox) => self.mailbox_action(mailbox),
            Action::Draft(draft) => self.draft_action(draft),
        }
    }

    /// App-wide actions.
    fn general(&mut self, general: General) {
        match general {
            General::Quit => self.quit = true,
            General::Help => self.overlay = Some(Overlay::Help),
            General::Refresh => self.load_envelopes(),
            General::Search => self.prompt_search(),
            General::Accounts => self.request("Listing accounts", Request::Accounts),
            General::ToggleThreads => self.toggle_threads(),
            General::ToggleCompact => self.toggle_compact(),
            General::Back => self.back(),
        }
    }
}
