//! The mail operations against the demo's pimdir store: reads come from the store, writes queue
//! for a sync engine but show at once.

use super::{
    ComposeKind, Draft, FlagChange, Keyword, MailClient, MessageRef, PageQuery, ReplyScope,
    Selection, Submit, Template, Transfer,
};
use crate::demo::{self, Store};

/// A seeded pimdir store in a temporary directory, and a client over it.
fn pimdir_client() -> (tempfile::TempDir, MailClient) {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let config = demo::create(dir.path(), Store::Pimdir).expect("the demo store");
    (dir, MailClient::new(vec![config], Some("demo".into())))
}

/// The id of the inbox message with `subject`.
fn inbox_id(client: &mut MailClient, subject: &str) -> String {
    let query = PageQuery {
        size: 200,
        ..PageQuery::first("imap/INBOX".into(), None)
    };
    let page = client.list_envelopes(&query).expect("the inbox");
    let envelope = page
        .envelopes
        .into_iter()
        .find(|e| e.subject.as_deref() == Some(subject));
    envelope.expect("the demo message").id
}

#[test]
fn a_store_lists_reads_and_searches() {
    let (_dir, mut client) = pimdir_client();
    let id = inbox_id(&mut client, "Weekend hike?");

    let message = client
        .read_message(&MessageRef {
            mailbox: "imap/INBOX".into(),
            id,
        })
        .expect("a read");
    let query = PageQuery::first("imap/INBOX".into(), Some("from:ada".into()));
    let found = client.list_envelopes(&query).expect("a search");

    assert!(
        message
            .text
            .is_some_and(|text| text.contains("ridge trail"))
    );
    assert_eq!(found.envelopes.len(), 1);
}

#[test]
fn writes_queue_and_show_at_once() {
    let (_dir, mut client) = pimdir_client();
    let id = inbox_id(&mut client, "Weekend hike?");
    let selection = Selection {
        mailbox: "imap/INBOX".into(),
        ids: vec![id],
    };

    let flagged = client
        .set_flag(&selection, Keyword::Flagged, FlagChange::Add)
        .expect("a flag");
    let transfer = Transfer {
        from: selection,
        to: "imap/Archive".into(),
    };
    let moved = client.move_messages(&transfer).expect("a move");

    assert_eq!((flagged.queued, moved.queued), (true, true));
    let query = PageQuery::first("imap/Archive".into(), None);
    let archive = client.list_envelopes(&query).expect("the archive");
    assert!(
        archive
            .envelopes
            .iter()
            .any(|e| e.subject.as_deref() == Some("Weekend hike?"))
    );
}

#[test]
fn a_send_queues_for_the_sync_engine() {
    let (_dir, mut client) = pimdir_client();
    let id = inbox_id(&mut client, "Weekend hike?");
    let source = MessageRef {
        mailbox: "imap/INBOX".into(),
        id,
    };
    let kind = ComposeKind::Reply {
        source,
        scope: ReplyScope::Sender,
    };
    let template = client.template(&kind).expect("a reply template");
    let mut draft = Draft::from_template(ComposeKind::New, Template::default());
    draft.to = vec!["rae@nitidus.test".into()];

    let receipt = client.submit(&draft, Submit::Send).expect("a queued send");

    assert_eq!(template.subject.as_deref(), Some("Re: Weekend hike?"));
    assert!(receipt.queued);
}
