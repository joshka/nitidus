//! The mail operations against the demo's Maildir account and SMTP sink.

use std::path::Path;

use super::{
    ComposeKind, Draft, Envelope, FlagChange, Keyword, MailClient, MessageRef, PageQuery,
    ReplyScope, Selection, Submit, Transfer,
};
use crate::demo::{self, Store};

/// A demo account in a temporary directory, and a client over it.
fn demo_client() -> (tempfile::TempDir, MailClient) {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let config = demo::create(dir.path(), Store::Maildir).expect("the demo account");
    (dir, MailClient::new(vec![config], Some("demo".into())))
}

/// Every envelope of `mailbox`, across pages.
fn all(client: &mut MailClient, mailbox: &str) -> Vec<Envelope> {
    let mut query = PageQuery::first(mailbox.into(), None);
    let mut envelopes = Vec::new();
    loop {
        let page = client.list_envelopes(&query).expect("a page");
        envelopes.extend(page.envelopes);
        if page.next_page.is_none() {
            return envelopes;
        }
        query = query.at_page(query.page + 1);
    }
}

/// The envelope of `mailbox` with `subject`, if any.
fn find(client: &mut MailClient, mailbox: &str, subject: &str) -> Option<Envelope> {
    all(client, mailbox)
        .into_iter()
        .find(|e| e.subject.as_deref() == Some(subject))
}

/// The message reference of an envelope found by subject.
fn reference(client: &mut MailClient, mailbox: &str, subject: &str) -> MessageRef {
    let envelope = find(client, mailbox, subject).expect("the demo message");
    MessageRef {
        mailbox: mailbox.into(),
        id: envelope.id,
    }
}

#[test]
fn reading_marks_seen_and_flags_and_moves_apply() {
    let (_dir, mut client) = demo_client();
    let message = reference(&mut client, "INBOX", "Weekend hike?");

    let read = client.read_message(&message).expect("the message");
    let selection = Selection {
        mailbox: "INBOX".into(),
        ids: vec![message.id.clone()],
    };
    client
        .set_flag(&selection, Keyword::Flagged, FlagChange::Remove)
        .expect("a flag change");
    let transfer = Transfer {
        from: selection,
        to: "Archive".into(),
    };
    client.move_messages(&transfer).expect("a move");

    assert_eq!(read.headers.subject.as_deref(), Some("Weekend hike?"));
    let archived = find(&mut client, "Archive", "Weekend hike?").expect("the archived message");
    assert_eq!(
        (archived.has(Keyword::Seen), archived.has(Keyword::Flagged)),
        (true, false)
    );
}

#[test]
fn the_inbox_spans_pages() {
    let (_dir, mut client) = demo_client();

    let first = client
        .list_envelopes(&PageQuery::first("INBOX".into(), None))
        .expect("page 1");
    let last = client
        .list_envelopes(&PageQuery::first("INBOX".into(), None).at_page(4))
        .expect("page 4");

    assert_eq!(
        (first.envelopes.len(), first.next_page.is_some()),
        (50, true)
    );
    assert_eq!(last.next_page, None);
}

#[test]
fn a_reply_to_self_arrives_threaded_with_the_quote_once() {
    let (dir, mut client) = demo_client();
    let source = reference(&mut client, "INBOX", "Weekend hike?");

    let receipt = reply_to_self(&mut client, source, "Count me in!");

    assert!(receipt.errors.is_empty(), "{:?}", receipt.errors);
    assert_eq!(delivered(dir.path()), 1);
    let reply = find(&mut client, "INBOX", "Re: Weekend hike?").expect("the delivered reply");
    let parent = find(&mut client, "INBOX", "Weekend hike?").expect("the parent");
    assert_eq!(
        reply.in_reply_to,
        parent.message_id.iter().cloned().collect::<Vec<_>>()
    );
    assert!(parent.has(Keyword::Answered));
    assert_eq!(quoted_times(&mut client, reply.id, "Thinking of doing"), 1);
    assert!(find(&mut client, "Sent", "Re: Weekend hike?").is_some());
}

/// Replies to `source`, addressed back to the demo account, `text` above the quote.
fn reply_to_self(client: &mut MailClient, source: MessageRef, text: &str) -> super::Receipt {
    let kind = ComposeKind::Reply {
        source,
        scope: ReplyScope::Sender,
    };
    let template = client.template(&kind).expect("a template");
    let mut draft = Draft::from_template(kind, template);
    draft.to = vec!["Rae Field <rae@nitidus.test>".into()];
    draft.body = format!("{text}\n\n{}", draft.body);
    client.submit(&draft, Submit::Send).expect("a send")
}

/// How many times `phrase` appears in the inbox message `id`.
fn quoted_times(client: &mut MailClient, id: String, phrase: &str) -> usize {
    let message = client
        .read_message(&MessageRef {
            mailbox: "INBOX".into(),
            id,
        })
        .expect("the message");
    message.text.unwrap_or_default().matches(phrase).count()
}

/// How many messages the SMTP sink delivered to the demo inbox.
fn delivered(dir: &Path) -> usize {
    std::fs::read_dir(dir.join("mail/new"))
        .expect("the inbox's new/")
        .count()
}

#[test]
fn a_reopened_draft_replaces_itself() {
    let (_dir, mut client) = demo_client();
    let template = client.template(&ComposeKind::New).expect("a template");
    let mut draft = Draft::from_template(ComposeKind::New, template);
    draft.to = vec!["ada@analytical.test".into()];
    "Loop notes".clone_into(&mut draft.subject);
    client.submit(&draft, Submit::SaveDraft).expect("a save");
    let saved = reference(&mut client, "Drafts", "Loop notes");
    let kind = ComposeKind::Draft(saved);
    let mut reopened = Draft::from_template(
        kind.clone(),
        client.template(&kind).expect("the saved draft"),
    );

    "Loop notes v2".clone_into(&mut reopened.subject);
    client.submit(&reopened, Submit::SaveDraft).expect("a save");

    assert_eq!(reopened.to, ["ada@analytical.test"]);
    let subjects: Vec<_> = all(&mut client, "Drafts")
        .into_iter()
        .filter_map(|e| e.subject)
        .collect();
    assert!(
        subjects.contains(&"Loop notes v2".to_owned())
            && !subjects.contains(&"Loop notes".to_owned())
    );
}

#[test]
fn a_forward_carries_the_attachments() {
    let (_dir, mut client) = demo_client();
    let source = reference(&mut client, "INBOX", "Difference engine no. 2 - drawings");
    let kind = ComposeKind::Forward(source);

    let draft = Draft::from_template(kind.clone(), client.template(&kind).expect("a template"));

    assert_eq!(draft.carried, ["engine-notes.pdf"]);
    assert_eq!(draft.subject, "Fwd: Difference engine no. 2 - drawings");
}

#[test]
fn attachments_download_to_the_given_directory() {
    let (dir, mut client) = demo_client();
    let message = reference(&mut client, "INBOX", "Difference engine no. 2 - drawings");
    let out = dir.path().join("downloads");
    std::fs::create_dir_all(&out).expect("the download directory");

    let paths = client
        .download_attachments(&message, &out)
        .expect("a download");

    assert_eq!(paths, [out.join("engine-notes.pdf").display().to_string()]);
}

#[test]
fn mailboxes_are_created_renamed_and_deleted() {
    let (_dir, mut client) = demo_client();

    client.create_mailbox("Receipts").expect("a create");
    client
        .rename_mailbox("Receipts", "Receipts 2026")
        .expect("a rename");
    let after_rename = mailbox_names(&mut client);
    client.delete_mailbox("Receipts 2026").expect("a delete");

    assert!(
        after_rename.contains(&"Receipts 2026".to_owned()),
        "{after_rename:?}"
    );
    assert!(!mailbox_names(&mut client).contains(&"Receipts 2026".to_owned()));
    assert_eq!(client.accounts().expect("the accounts"), ["demo", "work"]);
}

/// The names of the demo account's mailboxes.
fn mailbox_names(client: &mut MailClient) -> Vec<String> {
    client
        .list_mailboxes()
        .expect("the mailboxes")
        .into_iter()
        .map(|m| m.name)
        .collect()
}

#[test]
fn a_search_filters_by_sender() {
    let (_dir, mut client) = demo_client();

    let query = PageQuery::first("INBOX".into(), Some("from:ada".into()));
    let page = client.list_envelopes(&query).expect("a search");

    let subjects: Vec<_> = page
        .envelopes
        .iter()
        .map(Envelope::subject_or_placeholder)
        .collect();
    assert_eq!(subjects, ["Notes on the engine's loop construct"]);
}
