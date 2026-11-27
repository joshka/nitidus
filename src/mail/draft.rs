//! Writing mail: what a draft answers, the template it starts from, and how it is submitted.
//!
//! Himalaya builds every message. A draft starts from its JSON template, round-trips through the
//! user's editor as a plain file ([`mod@file`]), and goes back through the same command's flags
//! ([`argv`]).

mod argv;
mod file;
mod saved;

use std::path::PathBuf;

use serde::Deserialize;

pub use self::argv::template_argv;
use super::envelope::Address;
use super::selection::MessageRef;

/// What a draft answers, which decides the command that builds and sends it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposeKind {
    /// A new message.
    New,

    /// A reply to `source`.
    Reply {
        /// The message replied to.
        source: MessageRef,

        /// Who the reply goes to.
        scope: ReplyScope,
    },

    /// A forward of a message, carrying its attachments.
    Forward(MessageRef),

    /// A saved draft, opened again; what is sent or saved replaces it.
    Draft(MessageRef),
}

/// Who a reply goes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyScope {
    /// The sender alone.
    Sender,

    /// The sender and every other recipient.
    All,
}

/// Whether a draft goes out or into the drafts mailbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Submit {
    /// Send it, keeping a copy where the account says.
    Send,

    /// Save it to the drafts mailbox.
    SaveDraft,
}

/// The template `message compose|reply|forward` prints under `--json`.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Template {
    /// The sender, from the account.
    from: Option<Address>,

    /// The primary recipients.
    #[serde(default)]
    to: Vec<Address>,

    /// The carbon-copy recipients.
    #[serde(default)]
    cc: Vec<Address>,

    /// The blind carbon-copy recipients.
    #[serde(default)]
    bcc: Vec<Address>,

    /// The subject, `Re:` or `Fwd:` already added.
    pub subject: Option<String>,

    /// The body, the quote already laid out.
    #[serde(default)]
    pub body: String,

    /// The attachments the source brings along, named only.
    #[serde(default)]
    attachments: Vec<TemplateAttachment>,
}

/// An attachment a template names.
#[derive(Clone, Debug, Deserialize)]
struct TemplateAttachment {
    /// The file name, `None` when the part names none.
    filename: Option<String>,
}

/// A message being written: the template as the user edited it.
#[derive(Clone, Debug)]
pub struct Draft {
    /// What the draft answers.
    pub kind: ComposeKind,

    /// The sender, `None` for the account's address.
    pub from: Option<String>,

    /// The primary recipients, one address each.
    pub to: Vec<String>,

    /// The carbon-copy recipients.
    pub cc: Vec<String>,

    /// The blind carbon-copy recipients.
    pub bcc: Vec<String>,

    /// The subject.
    pub subject: String,

    /// The body, quote included, as it will be sent.
    pub body: String,

    /// Files to attach.
    pub attach: Vec<PathBuf>,

    /// Attachments the source brings along (a forward), named only.
    pub carried: Vec<String>,
}

impl Draft {
    /// A draft of `kind`, filled from its template.
    pub fn from_template(kind: ComposeKind, template: Template) -> Self {
        let addresses = |list: &[Address]| list.iter().map(ToString::to_string).collect();
        let carried = template
            .attachments
            .into_iter()
            .filter_map(|a| a.filename)
            .collect();
        Self {
            kind,
            from: template.from.as_ref().map(ToString::to_string),
            to: addresses(&template.to),
            cc: addresses(&template.cc),
            bcc: addresses(&template.bcc),
            subject: template.subject.unwrap_or_default(),
            body: template.body,
            attach: Vec::new(),
            carried,
        }
    }

    /// What stops the draft from being sent, if anything.
    pub fn problem(&self) -> Option<&'static str> {
        let no_recipients = self.to.is_empty() && self.cc.is_empty() && self.bcc.is_empty();
        no_recipients.then_some("no recipients")
    }

    /// Every attachment's file name: those carried from the source, then those added.
    pub fn attachment_names(&self) -> impl Iterator<Item = String> + '_ {
        let added = self.attach.iter().map(|path| {
            path.file_name()
                .map_or_default(|name| name.to_string_lossy().into_owned())
        });
        self.carried.iter().cloned().chain(added)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::rstest;

    use super::*;

    fn draft() -> Draft {
        Draft::from_template(ComposeKind::New, Template::default())
    }

    #[rstest]
    #[case::to(&["a@x.test"], &[], None)]
    #[case::bcc_alone(&[], &["b@x.test"], None)]
    #[case::nobody(&[], &[], Some("no recipients"))]
    fn a_draft_needs_someone_to_send_to(
        #[case] to: &[&str],
        #[case] bcc: &[&str],
        #[case] problem: Option<&str>,
    ) {
        let mut draft = draft();
        draft.to = to.iter().map(|&a| a.to_owned()).collect();
        draft.bcc = bcc.iter().map(|&a| a.to_owned()).collect();

        assert_eq!(draft.problem(), problem);
    }

    #[test]
    fn attachments_list_the_carried_ones_then_the_added() {
        let mut draft = draft();
        draft.carried = vec!["engine.pdf".into()];
        draft.attach = vec![PathBuf::from("/tmp/notes.txt")];

        assert_eq!(
            draft.attachment_names().collect::<Vec<_>>(),
            ["engine.pdf", "notes.txt"]
        );
    }

    #[test]
    fn a_template_fills_the_draft_with_addresses_written_out() {
        let json = serde_json::json!({
            "from": { "name": "Rae", "email": "rae@x.test" },
            "to": [{ "name": "Ada", "email": "ada@x.test" }],
            "subject": "Re: Hi",
            "body": "> hi\n",
            "attachments": [{ "filename": "a.pdf" }, { "filename": null }],
        });
        let template: Template = serde_json::from_value(json).expect("a template");

        let draft = Draft::from_template(ComposeKind::New, template);

        assert_eq!(
            (
                draft.from.as_deref(),
                draft.to,
                draft.subject,
                draft.carried
            ),
            (
                Some("Rae <rae@x.test>"),
                vec!["Ada <ada@x.test>".to_owned()],
                "Re: Hi".to_owned(),
                vec!["a.pdf".to_owned()]
            )
        );
    }
}
