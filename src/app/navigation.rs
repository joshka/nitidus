//! Moving around: focus between panes, selections within them, opening mailboxes and messages,
//! and paging through a mailbox.

use std::time::Instant;

use super::App;
use super::action::{FocusMove, ListAction};
use super::keymap::Context;
use super::overlay::{Overlay, PromptFor};
use crate::mail::{FlagChange, Keyword, MessageRef, PageQuery, Role};
use crate::ui::envelope_list::Loading;
use crate::ui::layout::PaneFocus;
use crate::ui::movement::{Movement, Navigate};
use crate::ui::overlay::Prompt;
use crate::worker::{Listing, ListingPurpose, Request};

/// The search prompt's hint: the pimdir query language, by example.
const SEARCH_HINT: &str = "from:ada  subject:engine  is:unread  has:attachment  date:2d";

/// The pane keys go to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    /// The mailbox list.
    Sidebar,

    /// The envelope list.
    List,

    /// The message reader.
    Reader,
}

impl Focus {
    /// The pane `movement` leads to from this one.
    fn moved(self, movement: FocusMove) -> Self {
        match (movement, self) {
            (FocusMove::Next | FocusMove::Right, Self::Sidebar)
            | (FocusMove::Previous | FocusMove::Left, Self::Reader) => Self::List,
            (FocusMove::Next | FocusMove::Right, Self::List)
            | (FocusMove::Previous, Self::Sidebar)
            | (FocusMove::Right, Self::Reader) => Self::Reader,
            (FocusMove::Next, Self::Reader)
            | (FocusMove::Previous | FocusMove::Left, Self::List)
            | (FocusMove::Left, Self::Sidebar) => Self::Sidebar,
        }
    }
}

impl App {
    /// Where keys are read now.
    pub(super) fn context(&self) -> Context {
        match (self.drafting.is_open(), self.focus) {
            (true, _) => Context::Draft,
            (false, Focus::Sidebar) => Context::Sidebar,
            (false, Focus::List) if self.list.has_marks() => Context::Marked,
            (false, Focus::List) => Context::List,
            (false, Focus::Reader) => Context::Reader,
        }
    }

    /// Moves focus between panes.
    pub(super) fn move_focus(&mut self, movement: FocusMove) {
        self.set_focus(self.focus.moved(movement));
    }

    /// Focuses `focus`, telling the sidebar and list whether they have it.
    pub(super) fn set_focus(&mut self, focus: Focus) {
        let pane = |pane: Focus| {
            if focus == pane {
                PaneFocus::Focused
            } else {
                PaneFocus::Unfocused
            }
        };
        self.focus = focus;
        self.sidebar.focus = pane(Focus::Sidebar);
        self.list.focus = pane(Focus::List);
    }

    /// Moves the focused pane's selection or scroll, opening what becomes selected.
    pub(super) fn move_focused(&mut self, movement: Movement) {
        if self.drafting.navigate(movement) {
            return;
        }
        match self.focus {
            Focus::Sidebar if self.sidebar.navigate(movement) => self.open_mailbox(),
            Focus::List if self.list.navigate(movement) => self.open_current(),
            Focus::Reader => drop(self.reader.navigate(movement)),
            Focus::Sidebar | Focus::List => {}
        }
    }

    /// Actions on the envelope list.
    pub(super) fn list_action(&mut self, action: ListAction) {
        match action {
            ListAction::Open => self.open_in_reader(),
            ListAction::ToggleMark => self.mark_and_advance(),
            ListAction::MarkAll => self.list.toggle_mark_all(),
            ListAction::NextMessage => self.step_message(Movement::Down),
            ListAction::PreviousMessage => self.step_message(Movement::Up),
        }
    }

    /// The open mailbox's id.
    pub(super) fn mailbox(&self) -> String {
        self.sidebar
            .current()
            .map_or_else(|| "INBOX".into(), |mailbox| mailbox.id.clone())
    }

    /// Whether the open mailbox plays `role`.
    pub(super) fn in_role(&self, role: Role) -> bool {
        self.sidebar
            .current()
            .is_some_and(|mailbox| mailbox.role() == role)
    }

    /// The selected message.
    pub(super) fn current_message(&self) -> Option<MessageRef> {
        let id = self.list.current()?.id.clone();
        Some(MessageRef {
            mailbox: self.mailbox(),
            id,
        })
    }

    /// Opens the sidebar's selected mailbox: an empty list, no search, then its first page.
    pub(super) fn open_mailbox(&mut self) {
        self.list.replace(Vec::new());
        self.search = None;
        self.reader.clear();
        self.load_envelopes();
    }

    /// Lists the open mailbox's first page, replacing the list.
    pub(super) fn load_envelopes(&mut self) {
        let mailbox = self.mailbox();
        self.list.title = self
            .sidebar
            .current()
            .map_or_else(|| mailbox.clone(), |m| m.label().to_owned());
        self.list.search.clone_from(&self.search);
        self.list.loading = Loading::FirstPage;
        self.last_check = Instant::now();
        let query = PageQuery::first(mailbox, self.search.clone());
        self.request(
            "Loading messages",
            Request::Envelopes(Listing {
                query,
                purpose: ListingPurpose::Open,
            }),
        );
    }

    /// Checks the open mailbox's first page for new mail, in the background.
    pub(super) fn check_for_mail(&mut self) {
        self.last_check = Instant::now();
        let query = PageQuery::first(self.mailbox(), None);
        self.request(
            "Checking for mail",
            Request::Envelopes(Listing {
                query,
                purpose: ListingPurpose::Check,
            }),
        );
    }

    /// Lists the next page when the selection nears the end of the list.
    fn load_more_if_near_end(&mut self) {
        let Some(page) = self
            .list
            .next_page
            .filter(|_| self.list.loading == Loading::Idle && self.list.near_end())
        else {
            return;
        };
        self.list.loading = Loading::NextPage;
        let query = PageQuery::first(self.mailbox(), self.search.clone()).at_page(page);
        self.request(
            "Loading more",
            Request::Envelopes(Listing {
                query,
                purpose: ListingPurpose::More,
            }),
        );
    }

    /// Reads the selected message into the reader, marking it seen in the list at once.
    pub(super) fn open_current(&mut self) {
        let Some(message) = self.current_message() else {
            self.reader.clear();
            return;
        };
        if self.reader.shows(&message.id) {
            return;
        }
        self.reader.clear();
        self.reader.loading = true;
        self.list
            .edit(std::slice::from_ref(&message.id), |envelope| {
                envelope.apply(Keyword::Seen, FlagChange::Add);
            });
        self.request("Reading", Request::Read(message));
        self.load_more_if_near_end();
    }

    /// Opens the selected message and focuses the reader.
    fn open_in_reader(&mut self) {
        self.open_current();
        self.set_focus(Focus::Reader);
    }

    /// Marks the selected message and moves to the next.
    fn mark_and_advance(&mut self) {
        self.list.toggle_mark();
        self.move_focused(Movement::Down);
    }

    /// Reads the next or previous message without leaving the reader.
    fn step_message(&mut self, movement: Movement) {
        if self.list.navigate(movement) {
            self.open_current();
        }
    }

    /// Switches between the date and thread views.
    pub(super) fn toggle_threads(&mut self) {
        let threaded = self.list.toggle_threads();
        self.info(if threaded { "Thread view" } else { "Date view" });
    }

    /// Steps back: closes the source, leaves the reader, clears marks, then clears the search.
    pub(super) fn back(&mut self) {
        if self.reader.showing_source() {
            self.reader.toggle_source();
        } else if self.focus == Focus::Reader {
            self.set_focus(Focus::List);
        } else if self.list.has_marks() {
            self.list.clear_marks();
        } else if self.search.take().is_some() {
            self.load_envelopes();
        }
    }

    /// Asks what to search for, starting from the active search.
    pub(super) fn prompt_search(&mut self) {
        let prompt = Prompt::new(
            "Search",
            self.search.clone().unwrap_or_default(),
            SEARCH_HINT,
        );
        self.overlay = Some(Overlay::Prompt(prompt, PromptFor::Search));
    }

    /// Searches the open mailbox for `query`, or lists it plainly when blank.
    pub(super) fn search_for(&mut self, query: String) {
        self.search = (!query.is_empty()).then_some(query);
        self.load_envelopes();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(FocusMove::Next, Focus::Sidebar, Focus::List)]
    #[case(FocusMove::Next, Focus::List, Focus::Reader)]
    #[case(FocusMove::Next, Focus::Reader, Focus::Sidebar)]
    #[case(FocusMove::Previous, Focus::Sidebar, Focus::Reader)]
    #[case(FocusMove::Previous, Focus::List, Focus::Sidebar)]
    #[case(FocusMove::Previous, Focus::Reader, Focus::List)]
    #[case(FocusMove::Left, Focus::Sidebar, Focus::Sidebar)]
    #[case(FocusMove::Left, Focus::List, Focus::Sidebar)]
    #[case(FocusMove::Left, Focus::Reader, Focus::List)]
    #[case(FocusMove::Right, Focus::Sidebar, Focus::List)]
    #[case(FocusMove::Right, Focus::List, Focus::Reader)]
    #[case(FocusMove::Right, Focus::Reader, Focus::Reader)]
    fn focus_moves_between_the_three_panes(
        #[case] movement: FocusMove,
        #[case] from: Focus,
        #[case] to: Focus,
    ) {
        assert_eq!(from.moved(movement), to);
    }
}
