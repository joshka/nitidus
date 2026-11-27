//! Everything a key can ask for, grouped by what it acts on. The keymap maps keys to these, and
//! each group is performed by the module that owns it.

use crate::ui::movement::Movement;

/// An action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// App-wide actions.
    General(General),

    /// Moving focus between panes.
    Focus(FocusMove),

    /// Moving the focused pane's selection or scroll.
    Move(Movement),

    /// Acting on the envelope list itself.
    List(ListAction),

    /// Triage: flags, archive, delete, move, copy.
    Triage(Triage),

    /// Starting a draft.
    Compose(ComposeAction),

    /// Reading beyond the text: attachments, links, source.
    Read(ReadAction),

    /// Managing mailboxes.
    Mailbox(MailboxAction),

    /// Acting on the draft under review.
    Draft(DraftAction),
}

/// App-wide actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum General {
    /// Quit.
    Quit,

    /// Show the key reference.
    Help,

    /// List the open mailbox again.
    Refresh,

    /// Search the open mailbox.
    Search,

    /// Switch account.
    Accounts,

    /// Switch between the date and thread views.
    ToggleThreads,

    /// Step back: close the source, leave the reader, clear marks, then clear the search.
    Back,
}

/// A focus move between the sidebar, the list and the reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusMove {
    /// The next pane, wrapping.
    Next,

    /// The previous pane, wrapping.
    Previous,

    /// The pane to the left.
    Left,

    /// The pane to the right.
    Right,
}

/// Actions on the envelope list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListAction {
    /// Open the selected message in the reader.
    Open,

    /// Mark the selected message and move on.
    ToggleMark,

    /// Mark every message, or clear the marks.
    MarkAll,

    /// Read the next message, from the reader.
    NextMessage,

    /// Read the previous message, from the reader.
    PreviousMessage,
}

/// Triage on the marked messages, else the selected one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Triage {
    /// Toggle read.
    ToggleSeen,

    /// Toggle the flag.
    ToggleFlag,

    /// Move to the archive mailbox.
    Archive,

    /// Move to the trash, or delete for good from inside it.
    Delete,

    /// Move to a mailbox picked from a list.
    MoveTo,

    /// Copy to a mailbox picked from a list.
    CopyTo,
}

/// Starting a draft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposeAction {
    /// A new message.
    New,

    /// Reply to the sender.
    Reply,

    /// Reply to everyone.
    ReplyAll,

    /// Forward.
    Forward,

    /// Open a saved draft again.
    EditDraft,
}

/// Reading beyond the message text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadAction {
    /// Open the attachments with the system's opener.
    OpenAttachments,

    /// Save the attachments to the downloads directory.
    SaveAttachments,

    /// Pick a link to open.
    Links,

    /// Open the HTML part in a browser.
    Browser,

    /// Show or hide the raw source.
    Source,
}

/// Managing mailboxes, from the sidebar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailboxAction {
    /// Create a mailbox.
    Create,

    /// Rename the open mailbox.
    Rename,

    /// Delete the open mailbox.
    Delete,
}

/// Acting on the draft under review.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftAction {
    /// Send it.
    Send,

    /// Save it to the drafts mailbox.
    Save,

    /// Edit it again.
    Edit,

    /// Attach a file.
    Attach,

    /// Discard it, after asking.
    Discard,
}
