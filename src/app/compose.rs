//! Writing mail: starting a draft, the editor round trip, and sending or saving it.

use anyhow::{Context as _, Result};
use ratatui::DefaultTerminal;

use super::App;
use super::action::{ComposeAction, DraftAction};
use super::overlay::{ConfirmFor, Overlay, PromptFor};
use crate::mail::{
    ComposeKind, Draft, FlagChange, Keyword, MessageRef, Receipt, ReplyScope, Role, Submit,
};
use crate::system;
use crate::ui::composer::Composer;
use crate::ui::movement::{Movement, Navigate};
use crate::ui::overlay::{Confirm, Prompt};
use crate::worker::Request;

/// The draft being written.
#[derive(Default)]
pub struct Drafting {
    /// The draft under review, `None` when none is open.
    composer: Option<Composer>,

    /// Whether the loop should open the draft in the editor.
    edit_requested: bool,

    /// Editor text that failed to parse, kept so editing again loses nothing.
    unparsed: Option<String>,
}

impl Drafting {
    /// Whether a draft is open.
    pub fn is_open(&self) -> bool {
        self.composer.is_some()
    }

    /// The draft under review.
    pub fn composer(&self) -> Option<&Composer> {
        self.composer.as_ref()
    }

    /// Scrolls the open draft, answering whether one is open.
    pub fn navigate(&mut self, movement: Movement) -> bool {
        self.composer
            .as_mut()
            .map(|composer| composer.navigate(movement))
            .is_some()
    }

    /// Whether the editor was asked for, clearing the request.
    pub fn take_edit_request(&mut self) -> bool {
        std::mem::take(&mut self.edit_requested)
    }
}

impl App {
    /// Starts a draft.
    pub(super) fn compose(&mut self, action: ComposeAction) {
        let kind = match action {
            ComposeAction::New => Some(ComposeKind::New),
            ComposeAction::Reply => self.reply_kind(ReplyScope::Sender),
            ComposeAction::ReplyAll => self.reply_kind(ReplyScope::All),
            ComposeAction::Forward => self.current_message().map(ComposeKind::Forward),
            ComposeAction::EditDraft => self
                .current_message()
                .filter(|_| self.in_role(Role::Drafts))
                .map(ComposeKind::Draft),
        };
        if let Some(kind) = kind {
            self.request("Preparing message", Request::Template(kind));
        }
    }

    /// A reply to the selected message.
    fn reply_kind(&self, scope: ReplyScope) -> Option<ComposeKind> {
        self.current_message()
            .map(|source| ComposeKind::Reply { source, scope })
    }

    /// Acts on the draft under review.
    pub(super) fn draft_action(&mut self, action: DraftAction) {
        match action {
            DraftAction::Send => self.send_draft(),
            DraftAction::Save => self.submit(Submit::SaveDraft),
            DraftAction::Edit => self.drafting.edit_requested = true,
            DraftAction::Attach => self.prompt_attachment(),
            DraftAction::Discard => self.confirm_discard(),
        }
    }

    /// A template arrived: open it under review, then in the editor.
    pub(super) fn draft_ready(&mut self, draft: Draft) {
        self.drafting.composer = Some(Composer::new(draft));
        self.drafting.edit_requested = true;
    }

    /// Hands the terminal to the editor and back; a draft that won't parse stays open with the
    /// problem shown.
    pub(super) fn edit_draft(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        ratatui::restore();
        let edited = self.run_editor();
        *terminal = ratatui::init();
        terminal.clear()?;
        if let Err(error) = edited {
            self.error(format!("{error:#}"));
        }
        Ok(())
    }

    /// Writes the draft to a file, runs the editor on it, and reads it back.
    fn run_editor(&mut self) -> Result<()> {
        let Some(composer) = self.drafting.composer.as_mut() else {
            return Ok(());
        };
        let file = tempfile::Builder::new()
            .prefix("nitidus-")
            .suffix(".eml")
            .tempfile()
            .context("create the draft file")?;
        let text = self
            .drafting
            .unparsed
            .take()
            .unwrap_or_else(|| composer.draft.to_file());
        std::fs::write(file.path(), &text).context("write the draft file")?;
        system::edit(file.path())?;
        let edited = std::fs::read_to_string(file.path()).context("read the draft file")?;
        let parsed = composer.draft.update_from_file(&edited);
        if parsed.is_err() {
            self.drafting.unparsed = Some(edited);
        }
        parsed.context("the draft has a problem (e to edit again)")
    }

    /// Sends the draft, unless it has a problem.
    fn send_draft(&mut self) {
        let problem = self
            .drafting
            .composer()
            .and_then(|composer| composer.draft.problem());
        match (self.drafting.unparsed.is_some(), problem) {
            (true, _) => self.error("Fix the draft first (e to edit)"),
            (false, Some(problem)) => self.error(format!("Can't send: {problem}")),
            (false, None) => self.submit(Submit::Send),
        }
    }

    /// Sends or saves the draft.
    fn submit(&mut self, submit: Submit) {
        let Some(composer) = self.drafting.composer() else {
            return;
        };
        let draft = Box::new(composer.draft.clone());
        let doing = if submit == Submit::Send {
            "Sending"
        } else {
            "Saving draft"
        };
        self.request(doing, Request::Submit(draft, submit));
    }

    /// The draft went out or was saved: close it, mark a replied message answered, and say so.
    pub(super) fn submitted(&mut self, submit: Submit, receipt: &Receipt) {
        let closed = self.drafting.composer.take();
        if let (Submit::Send, Some(source)) =
            (submit, closed.as_ref().and_then(|c| replied(&c.draft.kind)))
        {
            self.list
                .edit(std::slice::from_ref(&source.id), |envelope| {
                    envelope.apply(Keyword::Answered, FlagChange::Add);
                });
        }
        let said = match (submit, receipt.queued) {
            (Submit::Send, false) => "Message sent",
            (Submit::Send, true) => "Queued to send on the next sync",
            (Submit::SaveDraft, false) => "Saved to Drafts",
            (Submit::SaveDraft, true) => "Draft queued for the next sync",
        };
        self.report(said.to_owned(), receipt);
        if self.in_role(Role::Drafts) || self.in_role(Role::Sent) {
            self.load_envelopes();
        }
    }

    /// Asks for a file to attach.
    fn prompt_attachment(&mut self) {
        let prompt = Prompt::new("Attach file", "", "A path; ~ is your home");
        self.overlay = Some(Overlay::Prompt(prompt, PromptFor::Attach));
    }

    /// Attaches the file at `path` to the draft.
    pub(super) fn attach(&mut self, path: &str) {
        let file = system::expand_home(path);
        match self.drafting.composer.as_mut() {
            _ if !file.is_file() => self.error(format!("Not a file: {path}")),
            Some(composer) => composer.draft.attach.push(file),
            None => {}
        }
    }

    /// Asks before throwing the draft away.
    fn confirm_discard(&mut self) {
        self.overlay = Some(Overlay::Confirm(
            Confirm::new("Discard this message?"),
            ConfirmFor::Discard,
        ));
    }

    /// Throws the draft away.
    pub(super) fn discard_draft(&mut self) {
        self.drafting = Drafting::default();
        self.info("Discarded");
    }
}

/// The message a reply answers.
fn replied(kind: &ComposeKind) -> Option<&MessageRef> {
    match kind {
        ComposeKind::Reply { source, .. } => Some(source),
        ComposeKind::New | ComposeKind::Forward(_) | ComposeKind::Draft(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_reply_marks_its_source_answered() {
        let source = MessageRef {
            mailbox: "INBOX".into(),
            id: "1".into(),
        };
        let reply = ComposeKind::Reply {
            source: source.clone(),
            scope: ReplyScope::All,
        };

        assert_eq!(
            (replied(&reply), replied(&ComposeKind::Forward(source))),
            (
                Some(&MessageRef {
                    mailbox: "INBOX".into(),
                    id: "1".into()
                }),
                None
            )
        );
    }
}
