//! The Himalaya command lines that start a draft and submit it.
//!
//! Each is built one value per element with `--` before ids, so nothing is read as a flag or split
//! like a shell line.

use super::{ComposeKind, Draft, ReplyScope, Submit};

/// The command printing the template of a new message, a reply or a forward; `None` for a saved
/// draft, which is read raw instead.
pub fn template_argv(kind: &ComposeKind) -> Option<Vec<String>> {
    let mut argv = command_words(kind)?;
    argv.extend(words(&["--body", ""]));
    if let ComposeKind::Reply {
        scope: ReplyScope::All,
        ..
    } = kind
    {
        argv.push("--all".into());
    }
    argv.extend(source_id(kind));
    Some(argv)
}

impl Draft {
    /// The command sending or saving this draft, its body read from `body_path`.
    pub fn submit_argv(&self, body_path: &str, submit: Submit) -> Vec<String> {
        let mut argv = command_words(&self.kind).unwrap_or_else(|| words(&["message", "compose"]));
        argv.extend(self.header_flags(body_path));
        argv.extend(self.attach_flags());
        argv.extend(routing_flags(submit));
        argv.extend(source_id(&self.kind));
        argv
    }

    /// The sender, recipients, subject and body flags; replies and forwards say the quote is
    /// already laid out.
    fn header_flags(&self, body_path: &str) -> Vec<String> {
        let mut flags = Vec::new();
        for (flag, list) in [("--to", &self.to), ("--cc", &self.cc), ("--bcc", &self.bcc)] {
            flags.extend(
                list.iter()
                    .flat_map(|address| [flag.to_owned(), address.clone()]),
            );
        }
        if let Some(from) = &self.from {
            flags.extend(["--from".to_owned(), from.clone()]);
        }
        flags.extend(["--subject".to_owned(), self.subject.clone()]);
        flags.extend(["--body-file".to_owned(), body_path.to_owned()]);
        if source(&self.kind).is_some() {
            flags.extend(words(&["--posting-style", "none"]));
        }
        flags
    }

    /// One `--attach` per added file.
    fn attach_flags(&self) -> Vec<String> {
        let paths = self.attach.iter().map(|path| path.display().to_string());
        paths
            .flat_map(|path| ["--attach".to_owned(), path])
            .collect()
    }
}

/// `message compose`, `reply` or `forward` with its source mailbox; `None` for a saved draft.
fn command_words(kind: &ComposeKind) -> Option<Vec<String>> {
    let argv = match kind {
        ComposeKind::New => words(&["message", "compose"]),
        ComposeKind::Reply { source, .. } => words(&["message", "reply", "-m", &source.mailbox]),
        ComposeKind::Forward(source) => words(&["message", "forward", "-m", &source.mailbox]),
        ComposeKind::Draft(_) => return None,
    };
    Some(argv)
}

/// `--send`, or `--save drafts`.
fn routing_flags(submit: Submit) -> Vec<String> {
    match submit {
        Submit::Send => words(&["--send"]),
        Submit::SaveDraft => words(&["--save", "drafts"]),
    }
}

/// `-- <id>` for a reply or forward, nothing otherwise.
fn source_id(kind: &ComposeKind) -> Vec<String> {
    source(kind).map_or_default(|source| words(&["--", &source.id]))
}

/// The message a reply or forward is built from.
fn source(kind: &ComposeKind) -> Option<&super::MessageRef> {
    match kind {
        ComposeKind::Reply { source, .. } | ComposeKind::Forward(source) => Some(source),
        ComposeKind::New | ComposeKind::Draft(_) => None,
    }
}

/// Owned argv words.
fn words(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|&part| part.to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::super::{MessageRef, Template};
    use super::*;

    #[test]
    fn a_reply_submits_through_reply_with_the_quote_laid_out() {
        let source = MessageRef {
            mailbox: "INBOX".into(),
            id: "7".into(),
        };
        let kind = ComposeKind::Reply {
            source,
            scope: ReplyScope::Sender,
        };
        let mut draft = Draft::from_template(kind, Template::default());
        draft.to = vec!["ada@x.test".into()];
        draft.subject = "Re: hi".into();

        let argv = draft.submit_argv("/tmp/body", Submit::Send).join(" ");

        let expected = "message reply -m INBOX --to ada@x.test --subject Re: hi --body-file /tmp/body \
                        --posting-style none --send -- 7";
        assert_eq!(argv, expected);
    }
}
