//! What the app asks the worker to do, and the job that answers each request.

use std::path::PathBuf;

use anyhow::Result;

use super::response::{Response, Write};
use crate::mail::{
    ComposeKind, Draft, FlagChange, Keyword, MailClient, MessageRef, PageQuery, Selection, Submit,
    Transfer,
};

/// A request to the worker.
pub enum Request {
    /// List the configured accounts.
    Accounts,

    /// List the account's mailboxes.
    Mailboxes,

    /// One page of a listing or search.
    Envelopes(Listing),

    /// Read a message, marking it seen.
    Read(MessageRef),

    /// Read a message's raw source.
    Source(MessageRef),

    /// Set or clear a keyword.
    Flag(FlagUpdate),

    /// Move messages to another mailbox.
    Move(Transfer),

    /// Copy messages to another mailbox.
    Copy(Transfer),

    /// Move messages to the trash, or delete them there.
    Delete(Selection),

    /// Create a mailbox by name.
    CreateMailbox(String),

    /// Rename a mailbox.
    RenameMailbox(MailboxRename),

    /// Delete a mailbox by id.
    DeleteMailbox(String),

    /// Fetch the template a draft starts from.
    Template(ComposeKind),

    /// Send or save a draft.
    Submit(Box<Draft>, Submit),

    /// Write a message's attachments to a directory.
    Download(Download),
}

/// A page to list, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    /// The page.
    pub query: PageQuery,

    /// What the app does with it.
    pub purpose: ListingPurpose,
}

/// What the app does with a listed page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListingPurpose {
    /// Replace the list: a mailbox opened, a search run, a refresh asked for.
    Open,

    /// Append a later page as the selection nears the end.
    More,

    /// Add new mail found by the periodic check, saying how much arrived.
    Check,
}

/// A keyword set or cleared on a selection.
pub struct FlagUpdate {
    /// The messages.
    pub selection: Selection,

    /// The keyword.
    pub keyword: Keyword,

    /// Set or clear.
    pub change: FlagChange,
}

/// A mailbox renamed.
pub struct MailboxRename {
    /// The mailbox's id.
    pub mailbox: String,

    /// Its new name.
    pub name: String,
}

/// Attachments written to a directory, and what follows.
pub struct Download {
    /// The message carrying them.
    pub message: MessageRef,

    /// Where they go.
    pub dir: PathBuf,

    /// What the app does with them once written.
    pub then: AfterDownload,
}

/// What the app does with downloaded attachments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterDownload {
    /// Open them with the system's opener.
    Open,

    /// Say where they were saved.
    Report,
}

impl Request {
    /// Runs the request over the session and answers it.
    pub fn run(self, client: &mut MailClient) -> Response {
        match self {
            Self::Accounts => answer("list accounts", client.accounts(), Response::Accounts),
            Self::Mailboxes => mailboxes(client),
            Self::Envelopes(listing) => envelopes(client, listing),
            Self::Read(message) => read(client, &message),
            Self::Source(message) => source(client, message),
            Self::Flag(update) => flag(client, &update),
            Self::Move(transfer) => moved(client, &transfer),
            Self::Copy(transfer) => copied(client, &transfer),
            Self::Delete(selection) => delete(client, &selection),
            Self::CreateMailbox(name) => create_mailbox(client, &name),
            Self::RenameMailbox(rename) => rename_mailbox(client, &rename),
            Self::DeleteMailbox(id) => delete_mailbox(client, &id),
            Self::Template(kind) => template(client, kind),
            Self::Submit(draft, how) => submit(client, &draft, how),
            Self::Download(job) => download(client, &job),
        }
    }
}

/// The response `respond` makes of `result`, or a failure naming `action`.
fn answer<T>(
    action: &'static str,
    result: Result<T>,
    respond: impl FnOnce(T) -> Response,
) -> Response {
    result.map_or_else(|error| Response::failed(action, &error), respond)
}

/// The mailboxes.
fn mailboxes(client: &mut MailClient) -> Response {
    answer(
        "list mailboxes",
        client.list_mailboxes(),
        Response::Mailboxes,
    )
}

/// One page, answered with the listing it answers.
fn envelopes(client: &mut MailClient, listing: Listing) -> Response {
    let page = client.list_envelopes(&listing.query);
    answer("list envelopes", page, |page| {
        Response::Envelopes(listing, page)
    })
}

/// A message's source as text, answered with the message it belongs to.
fn source(client: &mut MailClient, message: MessageRef) -> Response {
    let raw = client.read_source(&message);
    answer("read source", raw, |raw| {
        let text = String::from_utf8_lossy(&raw).replace("\r\n", "\n");
        Response::Source(message, text)
    })
}

/// A keyword change.
fn flag(client: &mut MailClient, update: &FlagUpdate) -> Response {
    let receipt = client.set_flag(&update.selection, update.keyword, update.change);
    answer("update flag", receipt, |receipt| {
        Response::Wrote(Write::Flag, receipt)
    })
}

/// A move, described for the status bar.
fn moved(client: &mut MailClient, transfer: &Transfer) -> Response {
    let write = Write::Move {
        count: transfer.from.ids.len(),
        to: transfer.to.clone(),
    };
    answer("move", client.move_messages(transfer), |receipt| {
        Response::Wrote(write, receipt)
    })
}

/// A copy, described for the status bar.
fn copied(client: &mut MailClient, transfer: &Transfer) -> Response {
    let write = Write::Copy {
        count: transfer.from.ids.len(),
        to: transfer.to.clone(),
    };
    answer("copy", client.copy_messages(transfer), |receipt| {
        Response::Wrote(write, receipt)
    })
}

/// A message, read.
fn read(client: &mut MailClient, message: &MessageRef) -> Response {
    let read = client.read_message(message);
    answer("read message", read, |message| {
        Response::Message(Box::new(message))
    })
}

/// A draft sent or saved.
fn submit(client: &mut MailClient, draft: &Draft, submit: Submit) -> Response {
    let action = match submit {
        Submit::Send => "send",
        Submit::SaveDraft => "save draft",
    };
    answer(action, client.submit(draft, submit), |receipt| {
        Response::Submitted(submit, receipt)
    })
}

/// A delete, which moves to the trash or deletes for good.
fn delete(client: &mut MailClient, selection: &Selection) -> Response {
    let write = Write::Delete {
        count: selection.ids.len(),
    };
    answer("delete", client.delete_messages(selection), |receipt| {
        Response::Wrote(write, receipt)
    })
}

/// A create.
fn create_mailbox(client: &mut MailClient, name: &str) -> Response {
    mailbox_change(client.create_mailbox(name), format!("Created {name}"))
}

/// A mailbox delete.
fn delete_mailbox(client: &mut MailClient, id: &str) -> Response {
    mailbox_change(client.delete_mailbox(id), "Deleted the mailbox".to_owned())
}

/// A rename.
fn rename_mailbox(client: &mut MailClient, rename: &MailboxRename) -> Response {
    let renamed = client.rename_mailbox(&rename.mailbox, &rename.name);
    mailbox_change(renamed, format!("Renamed to {}", rename.name))
}

/// A mailbox create, rename or delete, answered with what to say.
fn mailbox_change(result: Result<()>, said: String) -> Response {
    answer("change mailbox", result, |()| {
        Response::MailboxesChanged(said)
    })
}

/// A template, answered as a draft ready to edit.
fn template(client: &mut MailClient, kind: ComposeKind) -> Response {
    let template = client.template(&kind);
    answer("prepare message", template, |t| {
        Response::Template(Box::new(Draft::from_template(kind, t)))
    })
}

/// Attachments written, answered with their paths and what follows.
fn download(client: &mut MailClient, download: &Download) -> Response {
    let paths = client.download_attachments(&download.message, &download.dir);
    answer("download attachments", paths, |paths| {
        Response::Downloaded(paths, download.then)
    })
}
