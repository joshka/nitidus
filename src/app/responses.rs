//! Folding worker answers back into the screen.

use super::App;
use super::triage::plural;
use crate::mail::{DeleteAction, EnvelopePage, Mailbox, Message, MessageRef, Receipt};
use crate::ui::envelope_list::Loading;
use crate::worker::{Failure, Listing, ListingPurpose, Response, Write};

impl App {
    /// Folds in one answer.
    pub(super) fn on_response(&mut self, response: Response) {
        match response {
            Response::Accounts(accounts) => self.pick_account(accounts),
            Response::Mailboxes(mailboxes) => self.mailboxes_listed(mailboxes),
            Response::Envelopes(listing, page) => self.envelopes_listed(&listing, page),
            Response::Message(message) => self.message_read(*message),
            Response::Source(message, source) => self.source_read(&message, source),
            Response::Wrote(write, receipt) => self.wrote(&write, &receipt),
            Response::MailboxesChanged(said) => self.mailboxes_changed(said),
            Response::Template(draft) => self.draft_ready(*draft),
            Response::Submitted(submit, receipt) => self.submitted(submit, &receipt),
            Response::Downloaded(paths, then) => self.downloaded(paths, then),
            Response::Failed(failure) => self.failed(&failure),
        }
        self.sidebar.unread = self.list.unread();
    }

    /// The mailboxes arrived: open the one asked for on the command line, else keep the open one.
    fn mailboxes_listed(&mut self, mailboxes: Vec<Mailbox>) {
        self.sidebar.set_mailboxes(mailboxes);
        if let Some(name) = self.initial_mailbox.take() {
            self.sidebar.select_named(&name);
        }
        self.load_envelopes();
    }

    /// A page arrived; one for a mailbox or search no longer shown is dropped.
    fn envelopes_listed(&mut self, listing: &Listing, page: EnvelopePage) {
        let shown = Listing {
            query: crate::mail::PageQuery::first(self.mailbox(), self.search.clone()),
            ..listing.clone()
        };
        if !listing.query.same_listing(&shown.query) && listing.purpose != ListingPurpose::Check {
            return;
        }
        let next_page = page.next_page.is_some().then_some(listing.query.page + 1);
        match listing.purpose {
            ListingPurpose::Open => self.first_page(page, next_page),
            ListingPurpose::More => self.later_page(page, next_page),
            ListingPurpose::Check => self.new_mail(page),
        }
    }

    /// A first page replaces the list and reads its first message.
    fn first_page(&mut self, page: EnvelopePage, next_page: Option<usize>) {
        self.list.replace(page.envelopes);
        self.list.next_page = next_page;
        self.list.loading = Loading::Idle;
        self.reader.clear();
        self.open_current();
    }

    /// A later page is appended.
    fn later_page(&mut self, page: EnvelopePage, next_page: Option<usize>) {
        self.list.add(page.envelopes);
        self.list.next_page = next_page;
        self.list.loading = Loading::Idle;
    }

    /// A background check adds what is new and says how much arrived.
    fn new_mail(&mut self, page: EnvelopePage) {
        let arrived = self.list.add(page.envelopes);
        if arrived > 0 {
            self.info(format!("{arrived} new message{}", plural(arrived)));
        }
    }

    /// A message arrived; one no longer selected is dropped.
    fn message_read(&mut self, message: Message) {
        if self
            .list
            .current()
            .is_some_and(|envelope| envelope.id == message.id)
        {
            self.reader.show(message);
        }
    }

    /// A source arrived; one for a message no longer selected is dropped.
    fn source_read(&mut self, message: &MessageRef, source: String) {
        if self
            .list
            .current()
            .is_some_and(|envelope| envelope.id == message.id)
        {
            self.reader.set_source(source);
        }
    }

    /// A write finished: say what it did, and reload the list if any item failed.
    fn wrote(&mut self, write: &Write, receipt: &Receipt) {
        if let Some(said) = describe(write, receipt) {
            self.report(said, receipt);
        } else if !receipt.errors.is_empty() {
            self.report("Updated".into(), receipt);
        }
        if !receipt.errors.is_empty() {
            self.load_envelopes();
        }
    }

    /// Says what `said` reports, noting a queued write and naming each failed item.
    pub(super) fn report(&mut self, said: String, receipt: &Receipt) {
        let said = if receipt.queued && !said.contains("sync") {
            format!("{said} (queued for sync)")
        } else {
            said
        };
        if receipt.errors.is_empty() {
            self.info(said);
            return;
        }
        let failed: Vec<String> = receipt.errors.iter().map(ToString::to_string).collect();
        self.error(format!(
            "{said}, but {} failed: {}",
            failed.len(),
            failed.join("; ")
        ));
    }

    /// A request failed: stop every loading note and show the error with its causes.
    fn failed(&mut self, failure: &Failure) {
        self.list.loading = Loading::Idle;
        self.reader.loading = false;
        self.error(format!("{}: {}", failure.action, failure.error));
    }
}

/// What a write did, for the status bar; `None` for a flag change, which shows in the list.
fn describe(write: &Write, receipt: &Receipt) -> Option<String> {
    match write {
        Write::Flag => None,
        Write::Move { count, to } => {
            Some(format!("Moved {count} message{} to {to}", plural(*count)))
        }
        Write::Copy { count, to } => {
            Some(format!("Copied {count} message{} to {to}", plural(*count)))
        }
        Write::Delete { count } if receipt.action == Some(DeleteAction::MovedToTrash) => {
            Some(format!("Moved to trash: {count} message{}", plural(*count)))
        }
        Write::Delete { count } => Some(format!("Deleted {count} message{}", plural(*count))),
    }
}
