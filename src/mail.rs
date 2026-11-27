//! The mail side: himalaya's own commands run in-process over one session.
//!
//! Like himalaya-tui, nitidus couples to himalaya's versioned `--json`
//! contract rather than its Rust types: each operation builds an argv,
//! himalaya's clap grammar parses it, and the printed JSON is decoded into
//! the small types below.

use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, FixedOffset};
use clap::Parser;
use himalaya::cli::Command;
use himalaya::session::{Session, SessionAccount};
use himalaya::unified::backend::Backend;
use pimalaya_cli::error::partial_outcome_of;
use pimalaya_cli::output::Bytes;
use pimalaya_cli::printer::Printer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One himalaya session kept for the whole run.
pub struct MailClient {
    session: Session<'static>,
}

impl MailClient {
    pub fn new(config_paths: Vec<PathBuf>, account: Option<String>) -> Self {
        // The session borrows these for its lifetime, which is the process.
        let config_paths: &'static [PathBuf] = Vec::leak(config_paths);
        let name = account.map(|name| &*String::leak(name));
        let session = Session {
            account: SessionAccount {
                config_paths,
                name,
                backend: Backend::Auto,
            },
            ..Default::default()
        };
        Self { session }
    }

    pub fn keep_alive(&mut self) -> Result<()> {
        self.session.keep_alive()
    }

    pub fn list_mailboxes(&mut self) -> Result<Vec<Mailbox>> {
        #[derive(Deserialize)]
        struct Listing {
            mailboxes: Vec<Mailbox>,
        }
        let listing: Listing = self.run(["mailbox", "list"])?.decode()?;
        Ok(listing.mailboxes)
    }

    /// One page of a mailbox (pages start at 1), or of a search over it.
    pub fn list_envelopes(
        &mut self,
        mailbox: &str,
        page: usize,
        size: usize,
        query: Option<&str>,
    ) -> Result<EnvelopePage> {
        let (page, size) = (page.to_string(), size.to_string());
        let mut argv = vec!["envelope"];
        match query {
            Some(query) => argv.extend([
                "search", "-m", mailbox, "-p", &page, "-s", &size, "--", query,
            ]),
            None => argv.extend(["list", "-m", mailbox, "-p", &page, "-s", &size]),
        }
        self.run(argv)?.decode()
    }

    /// Reads a message, marking it seen.
    pub fn read_message(&mut self, mailbox: &str, id: &str) -> Result<Message> {
        self.run(["message", "read", "-m", mailbox, "--seen", "--", id])?
            .decode()
    }

    /// The raw RFC 5322 source, flags left alone.
    pub fn read_raw(&mut self, mailbox: &str, id: &str) -> Result<Vec<u8>> {
        self.run(["message", "read", "-m", mailbox, "--raw", "--", id])?
            .bytes
            .context("himalaya printed no message bytes")
    }

    pub fn set_flag(
        &mut self,
        mailbox: &str,
        ids: &[String],
        flag: &str,
        on: bool,
    ) -> Result<Receipt> {
        let verb = if on { "add" } else { "remove" };
        self.run_ids(&["flag", verb, "-m", mailbox, "-f", flag, "--"], ids)
    }

    pub fn move_messages(&mut self, from: &str, to: &str, ids: &[String]) -> Result<Receipt> {
        self.run_ids(&["message", "move", "-f", from, "-t", to, "--"], ids)
    }

    pub fn copy_messages(&mut self, from: &str, to: &str, ids: &[String]) -> Result<Receipt> {
        self.run_ids(&["message", "copy", "-f", from, "-t", to, "--"], ids)
    }

    /// Moves to the trash, or deletes for good when already there.
    pub fn delete_messages(&mut self, mailbox: &str, ids: &[String]) -> Result<Receipt> {
        self.run_ids(&["message", "delete", "-m", mailbox, "--"], ids)
    }

    pub fn create_mailbox(&mut self, name: &str) -> Result<()> {
        self.run(["mailbox", "create", "--", name])?;
        Ok(())
    }

    pub fn rename_mailbox(&mut self, mailbox: &str, name: &str) -> Result<()> {
        self.run(["mailbox", "rename", "--", mailbox, name])?;
        Ok(())
    }

    pub fn delete_mailbox(&mut self, mailbox: &str) -> Result<()> {
        self.run(["mailbox", "delete", "--", mailbox])?;
        Ok(())
    }

    /// The editable template of a new message, a reply or a forward.
    pub fn template(&mut self, kind: &ComposeKind) -> Result<Template> {
        let argv: Vec<&str> = match kind {
            ComposeKind::New => vec!["message", "compose", "--body", ""],
            ComposeKind::Reply { mailbox, id, all } => {
                let mut v = vec!["message", "reply", "-m", mailbox, "--body", ""];
                if *all {
                    v.push("--all");
                }
                v.extend(["--", id]);
                v
            }
            ComposeKind::Forward { mailbox, id } => {
                vec!["message", "forward", "-m", mailbox, "--body", "", "--", id]
            }
            ComposeKind::Draft { mailbox, id } => {
                let raw = self.read_raw(mailbox, id)?;
                return Template::from_raw(&raw);
            }
        };
        self.run(argv)?.decode()
    }

    /// Sends the draft, or saves it to the drafts mailbox when `send` is false.
    pub fn submit(&mut self, draft: &Draft, send: bool) -> Result<Receipt> {
        let body = tempfile::NamedTempFile::new()?;
        std::fs::write(body.path(), &draft.body)?;
        let body_path = body
            .path()
            .to_str()
            .context("temporary path is not UTF-8")?
            .to_string();
        let mut argv: Vec<String> = match &draft.kind {
            ComposeKind::New | ComposeKind::Draft { .. } => {
                vec!["message".into(), "compose".into()]
            }
            ComposeKind::Reply { mailbox, .. } => vec![
                "message".into(),
                "reply".into(),
                "-m".into(),
                mailbox.clone(),
            ],
            ComposeKind::Forward { mailbox, .. } => vec![
                "message".into(),
                "forward".into(),
                "-m".into(),
                mailbox.clone(),
            ],
        };
        for (flag, list) in [
            ("--to", &draft.to),
            ("--cc", &draft.cc),
            ("--bcc", &draft.bcc),
        ] {
            for addr in list {
                argv.extend([flag.into(), addr.clone()]);
            }
        }
        if let Some(from) = &draft.from {
            argv.extend(["--from".into(), from.clone()]);
        }
        argv.extend([
            "--subject".into(),
            draft.subject.clone(),
            "--body-file".into(),
            body_path,
        ]);
        if matches!(
            draft.kind,
            ComposeKind::Reply { .. } | ComposeKind::Forward { .. }
        ) {
            // The body already holds the quote as the user left it.
            argv.extend(["--posting-style".into(), "none".into()]);
        }
        for path in &draft.attach {
            argv.extend(["--attach".into(), path.display().to_string()]);
        }
        if send {
            argv.push("--send".into());
        } else {
            argv.extend(["--save".into(), "drafts".into()]);
        }
        if let ComposeKind::Reply { id, .. } | ComposeKind::Forward { id, .. } = &draft.kind {
            argv.extend(["--".into(), id.clone()]);
        }
        let receipt: Receipt = self.run(argv.iter().map(String::as_str))?.decode()?;
        // A draft opened from the drafts mailbox is replaced by what was saved or sent.
        if let ComposeKind::Draft { mailbox, id } = &draft.kind {
            let _ = self.delete_messages(mailbox, std::slice::from_ref(id));
        }
        Ok(receipt)
    }

    /// Writes attachments to `dir`, returning the paths written.
    pub fn download_attachments(
        &mut self,
        mailbox: &str,
        id: &str,
        dir: &std::path::Path,
    ) -> Result<Vec<String>> {
        let dir = dir.to_str().context("download directory is not UTF-8")?;
        let printed = self.run(["attachment", "download", "-m", mailbox, "-d", dir, "--", id])?;
        let mut paths = vec![];
        collect_paths(printed.value.as_ref(), &mut paths);
        Ok(paths)
    }

    pub fn accounts(&mut self) -> Result<Vec<String>> {
        let value = self.run(["account", "list"])?.value.unwrap_or_default();
        let list = value.get("accounts").cloned().unwrap_or(value);
        Ok(list
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.get("name")?.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default())
    }

    fn run_ids(&mut self, argv: &[&str], ids: &[String]) -> Result<Receipt> {
        let argv = argv.iter().copied().chain(ids.iter().map(String::as_str));
        self.run(argv)?.decode()
    }

    fn run<'a>(&mut self, argv: impl IntoIterator<Item = &'a str>) -> Result<Captured> {
        let line = Line::try_parse_from(argv).map_err(|err| anyhow!("{err}"))?;
        let mut printer = Captured::default();
        match line.cmd.execute(&mut printer, &mut self.session) {
            Ok(()) => Ok(printer),
            Err(err) if partial_outcome_of(&err).is_some() => Ok(printer),
            Err(err) => Err(err),
        }
    }
}

/// A himalaya command line without the binary name or global flags.
#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
struct Line {
    #[command(subcommand)]
    cmd: Command,
}

/// What a command printed.
#[derive(Debug, Default)]
struct Captured {
    value: Option<Value>,
    bytes: Option<Vec<u8>>,
}

impl Captured {
    fn decode<T: DeserializeOwned>(self) -> Result<T> {
        let value = self.value.context("himalaya printed no result")?;
        serde_json::from_value(value).context("decode himalaya's answer")
    }
}

impl Printer for Captured {
    fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
        self.value = Some(serde_json::to_value(&data)?);
        Ok(())
    }
    fn event<T: fmt::Display + Serialize>(&mut self, _event: T) -> Result<()> {
        Ok(())
    }
    fn out_bytes(&mut self, bytes: Bytes) -> Result<()> {
        self.bytes = Some(bytes.data);
        Ok(())
    }
    fn is_json(&self) -> bool {
        true
    }
}

fn collect_paths(value: Option<&Value>, out: &mut Vec<String>) {
    match value {
        Some(Value::Object(map)) => {
            if let Some(Value::String(p)) = map.get("path") {
                out.push(p.clone());
            }
            for v in map.values() {
                collect_paths(Some(v), out);
            }
        }
        Some(Value::Array(list)) => list.iter().for_each(|v| collect_paths(Some(v), out)),
        _ => {}
    }
}

/// The answer of a write acting on messages by id.
#[derive(Debug, Default, Deserialize)]
pub struct Receipt {
    /// Whether a sync engine applies the write later (pimdir).
    #[serde(default)]
    pub queued: bool,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub errors: Vec<ItemError>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ItemError {
    pub id: Option<String>,
    pub error: String,
}

impl fmt::Display for ItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.id {
            Some(id) => write!(f, "{id}: {}", self.error),
            None => f.write_str(&self.error),
        }
    }
}

/// What a draft answers, which decides the command that sends it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposeKind {
    New,
    Reply {
        mailbox: String,
        id: String,
        all: bool,
    },
    Forward {
        mailbox: String,
        id: String,
    },
    /// A saved draft, opened again.
    Draft {
        mailbox: String,
        id: String,
    },
}

/// The `message compose|reply|forward` template.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub from: Option<Address>,
    #[serde(default)]
    pub to: Vec<Address>,
    #[serde(default)]
    pub cc: Vec<Address>,
    #[serde(default)]
    pub bcc: Vec<Address>,
    pub subject: Option<String>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub attachments: Vec<TemplateAttachment>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateAttachment {
    pub filename: Option<String>,
}

impl Template {
    /// A saved draft's headers and text, read back for editing.
    fn from_raw(raw: &[u8]) -> Result<Self> {
        let text = String::from_utf8_lossy(raw).replace("\r\n", "\n");
        let (head, body) = text.split_once("\n\n").unwrap_or((&text, ""));
        let mut t = Template {
            body: body.to_string(),
            ..Default::default()
        };
        for line in head.lines() {
            let Some((k, v)) = line.split_once(':') else {
                continue;
            };
            let v = v.trim();
            let addrs = || {
                crate::compose::split_addresses(v)
                    .into_iter()
                    .map(|e| Address {
                        name: None,
                        email: e,
                    })
                    .collect()
            };
            match k.to_ascii_lowercase().as_str() {
                "from" => {
                    t.from = Some(Address {
                        name: None,
                        email: v.to_string(),
                    });
                }
                "to" => t.to = addrs(),
                "cc" => t.cc = addrs(),
                "bcc" => t.bcc = addrs(),
                "subject" => t.subject = Some(v.to_string()),
                _ => {}
            }
        }
        Ok(t)
    }
}

/// A message being written: the template as the user edited it.
#[derive(Clone, Debug)]
pub struct Draft {
    pub kind: ComposeKind,
    pub from: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub attach: Vec<PathBuf>,
    /// Attachments the source brings along (forward), named only.
    pub carried: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Mailbox {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
}

impl Mailbox {
    /// The name to show: pimdir names a mailbox after its source
    /// (`imap/Archive`), which the label leaves out.
    pub fn label(&self) -> &str {
        self.name.rsplit('/').next().unwrap_or(&self.name)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopePage {
    pub envelopes: Vec<Envelope>,
    pub next_page: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    pub id: String,
    #[serde(default)]
    pub flags: BTreeSet<Flag>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub from: Vec<Address>,
    #[serde(default)]
    pub date: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub has_attachment: Option<bool>,
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub in_reply_to: Vec<String>,
}

impl Envelope {
    pub fn has(&self, iana: &str) -> bool {
        self.flags.iter().any(|f| f.iana.as_deref() == Some(iana))
    }
    pub fn set(&mut self, iana: &str, on: bool) {
        self.flags.retain(|f| f.iana.as_deref() != Some(iana));
        if on {
            self.flags.insert(Flag {
                raw: iana.into(),
                iana: Some(iana.into()),
            });
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flag {
    pub raw: String,
    pub iana: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Address {
    pub name: Option<String>,
    pub email: String,
}

impl Address {
    pub fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or(&self.email)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) if !name.is_empty() => write!(f, "{name} <{}>", self.email),
            _ => f.write_str(&self.email),
        }
    }
}

/// The `message read` answer.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub headers: Headers,
    pub text: Option<String>,
    pub html: Option<String>,
    #[serde(default)]
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Headers {
    #[serde(default)]
    pub from: Vec<Address>,
    #[serde(default)]
    pub to: Vec<Address>,
    #[serde(default)]
    pub cc: Vec<Address>,
    pub subject: Option<String>,
    pub date: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub mime: Option<String>,
}

impl Message {
    /// The body as plain text, rendering HTML when there is no text part.
    pub fn body(&self, width: usize) -> String {
        if let Some(text) = &self.text {
            return text.replace("\r\n", "\n");
        }
        if let Some(html) = &self.html {
            return html2text::from_read(html.as_bytes(), width.max(20))
                .unwrap_or_else(|_| html.clone());
        }
        String::new()
    }

    pub fn attachments(&self) -> impl Iterator<Item = &str> {
        self.parts.iter().filter_map(|p| p.filename.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::MailClient;

    fn demo() -> (tempfile::TempDir, MailClient) {
        let dir = tempfile::tempdir().unwrap();
        let config = crate::demo::create(dir.path(), false).unwrap();
        (dir, MailClient::new(vec![config], Some("demo".into())))
    }

    #[test]
    fn demo_account_lists_reads_flags_and_archives() {
        let (_dir, mut c) = demo();
        let names: Vec<_> = c
            .list_mailboxes()
            .unwrap()
            .into_iter()
            .map(|m| m.name)
            .collect();
        assert!(names.iter().any(|n| n == "Archive"), "{names:?}");

        let page = c.list_envelopes("INBOX", 1, 100, None).unwrap();
        assert!(page.envelopes.len() > 10);
        let e = page
            .envelopes
            .iter()
            .find(|e| !e.has("seen"))
            .unwrap()
            .clone();

        let m = c.read_message("INBOX", &e.id).unwrap();
        assert_eq!(m.headers.subject, e.subject);
        assert_ne!(m.body(80), "");

        c.set_flag("INBOX", std::slice::from_ref(&e.id), "flagged", true)
            .unwrap();
        let after = c.list_envelopes("INBOX", 1, 100, None).unwrap();
        let e2 = after
            .envelopes
            .iter()
            .find(|x| x.subject == e.subject)
            .unwrap();
        assert!(e2.has("flagged") && e2.has("seen"), "{:?}", e2.flags);

        c.move_messages("INBOX", "Archive", std::slice::from_ref(&e2.id))
            .unwrap();
        let archive = c.list_envelopes("Archive", 1, 100, None).unwrap();
        assert!(archive.envelopes.iter().any(|x| x.subject == e.subject));
    }

    fn find(c: &mut MailClient, mailbox: &str, subject: &str) -> Option<super::Envelope> {
        let page = c.list_envelopes(mailbox, 1, 200, None).unwrap();
        page.envelopes
            .into_iter()
            .find(|e| e.subject.as_deref() == Some(subject))
    }

    #[test]
    fn inbox_pages() {
        let (_dir, mut c) = demo();
        let one = c.list_envelopes("INBOX", 1, 50, None).unwrap();
        assert_eq!(one.envelopes.len(), 50);
        assert!(one.next_page.is_some());
        let four = c.list_envelopes("INBOX", 4, 50, None).unwrap();
        assert!(
            four.envelopes.len() < 50 && four.next_page.is_none(),
            "{}",
            four.envelopes.len()
        );
    }

    #[test]
    fn reply_template_quotes_and_sends_to_self_with_threading() {
        let (dir, mut c) = demo();
        let src = find(&mut c, "INBOX", "Weekend hike?").unwrap();
        let kind = super::ComposeKind::Reply {
            mailbox: "INBOX".into(),
            id: src.id.clone(),
            all: false,
        };
        let t = c.template(&kind).unwrap();
        assert_eq!(t.subject.as_deref(), Some("Re: Weekend hike?"));
        assert!(
            t.body.contains("> Thinking of doing the ridge trail"),
            "{}",
            t.body
        );
        let mut draft = super::Draft::from_template(kind, t);
        draft.to = vec!["Rae Field <rae@nitidus.test>".into()];
        draft.body = format!("Count me in!\n\n{}", draft.body);
        let r = c.submit(&draft, true).unwrap();
        assert!(r.errors.is_empty(), "{:?}", r.errors);

        // The sink delivers mail for the demo address back to the inbox.
        let delivered = std::fs::read_dir(dir.path().join("mail/new"))
            .unwrap()
            .count();
        assert_eq!(delivered, 1);
        let got = find(&mut c, "INBOX", "Re: Weekend hike?").expect("delivered reply");
        assert!(
            got.in_reply_to
                .iter()
                .any(|id| src.message_id.as_ref() == Some(id)),
            "{got:?}"
        );
        let body = c.read_message("INBOX", &got.id).unwrap().text.unwrap();
        assert_eq!(
            body.matches("Thinking of doing").count(),
            1,
            "quoted once: {body}"
        );
        assert!(body.contains("Count me in!") && body.contains("Rae"));
        // A copy is kept in Sent, and the source is marked answered.
        assert!(find(&mut c, "Sent", "Re: Weekend hike?").is_some());
        assert!(
            find(&mut c, "INBOX", "Weekend hike?")
                .unwrap()
                .has("answered")
        );
    }

    #[test]
    fn draft_saves_then_reopens_and_replaces_itself() {
        let (_dir, mut c) = demo();
        let mut draft = super::Draft::from_template(
            super::ComposeKind::New,
            c.template(&super::ComposeKind::New).unwrap(),
        );
        draft.to = vec!["ada@analytical.test".into()];
        draft.subject = "Loop notes".into();
        draft.body = "Half written\n".into();
        c.submit(&draft, false).unwrap();
        let saved = find(&mut c, "Drafts", "Loop notes").expect("draft saved");

        let kind = super::ComposeKind::Draft {
            mailbox: "Drafts".into(),
            id: saved.id,
        };
        let mut reopened = super::Draft::from_template(kind.clone(), c.template(&kind).unwrap());
        assert_eq!(reopened.to, ["ada@analytical.test"]);
        reopened.subject = "Loop notes v2".into();
        c.submit(&reopened, false).unwrap();
        assert!(find(&mut c, "Drafts", "Loop notes").is_none());
        assert!(find(&mut c, "Drafts", "Loop notes v2").is_some());
    }

    #[test]
    fn forward_carries_attachments() {
        let (_dir, mut c) = demo();
        let src = find(&mut c, "INBOX", "Difference engine no. 2 - drawings").unwrap();
        let kind = super::ComposeKind::Forward {
            mailbox: "INBOX".into(),
            id: src.id.clone(),
        };
        let draft = super::Draft::from_template(kind, c.template(&kind_clone(&src.id)).unwrap());
        assert_eq!(draft.carried, ["engine-notes.pdf"]);
        assert_eq!(draft.subject, "Fwd: Difference engine no. 2 - drawings");
    }

    fn kind_clone(id: &str) -> super::ComposeKind {
        super::ComposeKind::Forward {
            mailbox: "INBOX".into(),
            id: id.into(),
        }
    }

    #[test]
    fn attachments_download_and_mailboxes_change() {
        let (dir, mut c) = demo();
        let src = find(&mut c, "INBOX", "Difference engine no. 2 - drawings").unwrap();
        let out = dir.path().join("dl");
        std::fs::create_dir_all(&out).unwrap();
        let paths = c.download_attachments("INBOX", &src.id, &out).unwrap();
        assert_eq!(paths.len(), 1, "{paths:?}");
        assert!(paths[0].ends_with("engine-notes.pdf"));

        c.create_mailbox("Receipts").unwrap();
        c.rename_mailbox("Receipts", "Receipts 2026").unwrap();
        let names: Vec<_> = c
            .list_mailboxes()
            .unwrap()
            .into_iter()
            .map(|m| m.name)
            .collect();
        assert!(names.contains(&"Receipts 2026".to_string()), "{names:?}");
        c.delete_mailbox("Receipts 2026").unwrap();
        assert_eq!(c.accounts().unwrap(), ["demo", "work"]);
    }

    #[test]
    fn search_filters_by_sender() {
        let (_dir, mut c) = demo();
        let page = c.list_envelopes("INBOX", 1, 100, Some("from:ada")).unwrap();
        assert!(!page.envelopes.is_empty());
        assert!(
            page.envelopes
                .iter()
                .all(|e| e.from[0].email.starts_with("ada"))
        );
    }
}

#[cfg(test)]
mod pimdir_tests {
    use super::*;

    fn store() -> (tempfile::TempDir, MailClient) {
        let dir = tempfile::tempdir().unwrap();
        let config = crate::demo::create(dir.path(), true).unwrap();
        (dir, MailClient::new(vec![config], Some("demo".into())))
    }

    /// Every operation nitidus runs, against a pimdir store, reporting what
    /// each answers rather than stopping at the first failure.
    #[test]
    fn pimdir_survey() {
        let (dir, mut c) = store();
        let mut report = vec![];
        let mut note = |what: &str, r: Result<String>| {
            report.push(format!(
                "{what:<28} {}",
                match r {
                    Ok(v) => format!("ok   {v}"),
                    Err(e) => format!("FAIL {e:#}"),
                }
            ));
        };
        let mbs = c.list_mailboxes();
        note(
            "mailbox list",
            mbs.as_ref()
                .map(|m| {
                    format!(
                        "{:?}",
                        m.iter()
                            .map(|m| (&m.id, &m.name, &m.role))
                            .collect::<Vec<_>>()
                    )
                })
                .map_err(|e| anyhow!("{e:#}")),
        );
        let page = c.list_envelopes("imap/INBOX", 1, 50, None);
        note(
            "envelope list p1",
            page.as_ref()
                .map(|p| format!("{} next={:?}", p.envelopes.len(), p.next_page))
                .map_err(|e| anyhow!("{e:#}")),
        );
        let p4 = c.list_envelopes("imap/INBOX", 4, 50, None);
        note(
            "envelope list p4",
            p4.map(|p| format!("{} next={:?}", p.envelopes.len(), p.next_page)),
        );
        let inbox = c.list_envelopes("inbox", 1, 200, None);
        note(
            "list via alias 'inbox'",
            inbox.map(|p| format!("{}", p.envelopes.len())),
        );
        let s = c.list_envelopes("imap/INBOX", 1, 50, Some("from:ada"));
        note(
            "search from:ada",
            s.map(|p| {
                format!(
                    "{:?}",
                    p.envelopes
                        .iter()
                        .map(|e| e.subject.clone())
                        .collect::<Vec<_>>()
                )
            }),
        );
        let s2 = c.list_envelopes("imap/INBOX", 1, 50, Some("engine"));
        note(
            "search text 'engine'",
            s2.map(|p| format!("{}", p.envelopes.len())),
        );

        let envs = page.map(|p| p.envelopes).unwrap_or_default();
        let find = |s: &str| {
            envs.iter()
                .find(|e| e.subject.as_deref() == Some(s))
                .cloned()
        };
        let hike = find("Weekend hike?");
        let babbage = find("Difference engine no. 2 - drawings");
        note(
            "threading fields",
            Ok(format!(
                "{:?}",
                envs.iter()
                    .find(|e| !e.in_reply_to.is_empty())
                    .map(|e| (&e.message_id, &e.in_reply_to))
            )),
        );
        if let Some(h) = &hike {
            note(
                "message read",
                c.read_message("imap/INBOX", &h.id)
                    .map(|m| format!("text={}", m.text.map_or(0, |t| t.len()))),
            );
            note(
                "message read --raw",
                c.read_raw("imap/INBOX", &h.id)
                    .map(|r| format!("{} bytes", r.len())),
            );
            note(
                "flag add flagged",
                c.set_flag("imap/INBOX", std::slice::from_ref(&h.id), "flagged", true)
                    .map(|r| format!("{r:?}")),
            );
            let after = c.list_envelopes("imap/INBOX", 1, 50, None).unwrap();
            let h2 = after.envelopes.iter().find(|e| e.id == h.id).map(|e| {
                format!(
                    "{:?}",
                    e.flags.iter().map(|f| f.raw.clone()).collect::<Vec<_>>()
                )
            });
            note("  flags visible after", Ok(format!("{h2:?}")));
            note(
                "reply template",
                c.template(&ComposeKind::Reply {
                    mailbox: "imap/INBOX".into(),
                    id: h.id.clone(),
                    all: false,
                })
                .map(|t| format!("subject={:?} quote={}", t.subject, t.body.contains("> "))),
            );
            let mut d = Draft::from_template(ComposeKind::New, Template::default());
            d.to = vec!["rae@nitidus.test".into()];
            d.subject = "pimdir send".into();
            d.body = "hi\n".into();
            note(
                "send (queued?)",
                c.submit(&d, true).map(|r| format!("{r:?}")),
            );
            note("save draft", c.submit(&d, false).map(|r| format!("{r:?}")));
            note(
                "move to Archive",
                c.move_messages("imap/INBOX", "imap/Archive", std::slice::from_ref(&h.id))
                    .map(|r| format!("{r:?}")),
            );
            let after = c.list_envelopes("imap/INBOX", 1, 200, None).unwrap();
            note(
                "  still in INBOX after?",
                Ok(format!("{}", after.envelopes.iter().any(|e| e.id == h.id))),
            );
            let arch = c.list_envelopes("imap/Archive", 1, 200, None).unwrap();
            note(
                "  in Archive after?",
                Ok(format!(
                    "{}",
                    arch.envelopes.iter().any(|e| e.subject == h.subject)
                )),
            );
        }
        if let Some(b) = &babbage {
            let out = dir.path().join("dl");
            std::fs::create_dir_all(&out).unwrap();
            note(
                "attachment download",
                c.download_attachments("imap/INBOX", &b.id, &out)
                    .map(|p| format!("{p:?}")),
            );
            note(
                "forward template",
                c.template(&ComposeKind::Forward {
                    mailbox: "imap/INBOX".into(),
                    id: b.id.clone(),
                })
                .map(|t| {
                    format!(
                        "{:?}",
                        t.attachments
                            .iter()
                            .map(|a| a.filename.clone())
                            .collect::<Vec<_>>()
                    )
                }),
            );
            note(
                "delete (to trash)",
                c.delete_messages("imap/INBOX", std::slice::from_ref(&b.id))
                    .map(|r| format!("{r:?}")),
            );
        }
        note(
            "mailbox create",
            c.create_mailbox("imap/Receipts").map(|()| String::new()),
        );
        note(
            "  listed after?",
            c.list_mailboxes()
                .map(|m| format!("{}", m.iter().any(|m| m.name.contains("Receipts")))),
        );
        note("accounts", c.accounts().map(|a| format!("{a:?}")));
        println!("\n{}\n", report.join("\n"));
        // A seeded store declares no sync capabilities, so only creating a
        // mailbox is refused; a store a sync engine fills declares them.
        let failed: Vec<&String> = report
            .iter()
            .filter(|l| l.contains("FAIL") && !l.starts_with("mailbox create"))
            .collect();
        assert!(failed.is_empty(), "{failed:#?}");
    }
}
