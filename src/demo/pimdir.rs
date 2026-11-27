//! Seeding a pimdir store from the demo's Maildir, as a sync engine's `imap` source fills one.
//!
//! Bodies are stored whole so messages can be read. No sync engine runs, so writes stay queued.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use io_pimdir::change::PimdirWriteOp;
use io_pimdir::client::PimdirStore;
use io_pimdir::collection::PimdirCollectionId;
use io_pimdir::object::PimdirHash;
use io_pimdir::placement::{
    PimdirBase, PimdirFlags, PimdirHandle, PimdirLevel, PimdirPlacement, PimdirStatus,
};
use io_pimdir::summary::mail;

/// The source the store is seeded as.
const SOURCE: &str = "imap";

/// The Maildir folders seeded, and the collection each becomes (`imap/<name>`).
const COLLECTIONS: [(&str, &str); 5] = [
    ("", "INBOX"),
    ("Archive", "Archive"),
    ("Drafts", "Drafts"),
    ("Sent", "Sent"),
    ("Trash", "Trash"),
];

/// The media type of a mail collection.
const MAIL_KIND: &str = "message/rfc822";

/// Seeds a store at `dir/pimdir` from the Maildir at `maildir`, returning the store's root.
pub fn seed(dir: &Path, maildir: &Path) -> Result<PathBuf> {
    let root = dir.join("pimdir");
    fs::create_dir_all(&root).with_context(|| format!("create {}", root.display()))?;
    let mut store = PimdirStore::open(&root)?.for_source(SOURCE)?;
    for (folder, name) in COLLECTIONS {
        let collection = format!("{SOURCE}/{name}");
        store.ensure_collection(&collection, MAIL_KIND)?;
        let ops = folder_ops(&store, &collection, &maildir.join(folder).join("cur"))?;
        store.write(ops)?;
    }
    Ok(root)
}

/// The writes filing every message of a Maildir `cur/` into `collection`.
fn folder_ops(store: &PimdirStore, collection: &str, cur: &Path) -> Result<Vec<PimdirWriteOp>> {
    let mut ops = Vec::new();
    for entry in fs::read_dir(cur).with_context(|| format!("read {}", cur.display()))? {
        let path = entry?.path();
        let body = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let file = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        ops.extend(message_ops(store, collection, file, &body)?);
    }
    Ok(ops)
}

/// The writes storing one message's body and filing it under its Maildir file name.
fn message_ops(
    store: &PimdirStore,
    collection: &str,
    file: String,
    body: &[u8],
) -> Result<[PimdirWriteOp; 2]> {
    let object = store.blob_stage().stage_body(body)?;
    let placed = placement(collection, file, body, object.hash.clone());
    Ok([
        PimdirWriteOp::StoreObject { object, body: None },
        PimdirWriteOp::UpsertPlacement(placed),
    ])
}

/// A message filed under `file` in `collection`, its body stored as `hash`, in sync with the
/// source.
fn placement(collection: &str, file: String, body: &[u8], hash: PimdirHash) -> PimdirPlacement {
    let derived = mail::derive(body);
    let flags = maildir_flags(&file);
    PimdirPlacement {
        collection: PimdirCollectionId(collection.to_owned()),
        handle: PimdirHandle(file),
        link_id: Some(derived.link_id),
        object: Some(hash),
        level: PimdirLevel::Full,
        summary: derived.summary,
        sort_key: derived.sort_key,
        server_object_id: None,
        flags: flags.clone(),
        status: PimdirStatus::Clean,
        conflict_revision: None,
        conflict_object: None,
        base: Some(PimdirBase {
            flags,
            revision: None,
            object: None,
        }),
        origin: None,
    }
}

/// The IMAP flags a Maildir file name's info letters stand for.
fn maildir_flags(file: &str) -> PimdirFlags {
    let letters = file.rsplit_once(":2,").map_or("", |(_, letters)| letters);
    PimdirFlags::from_iter(letters.chars().filter_map(imap_flag))
}

/// The IMAP flag of one Maildir info letter.
fn imap_flag(letter: char) -> Option<&'static str> {
    match letter {
        'S' => Some("\\Seen"),
        'F' => Some("\\Flagged"),
        'R' => Some("\\Answered"),
        'D' => Some("\\Draft"),
        _ => None,
    }
}
