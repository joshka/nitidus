//! The session every command of the run goes through, and how a command's output is captured.

use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
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

use super::draft::template_argv;
use super::{
    ComposeKind, Draft, EnvelopePage, FlagChange, Keyword, Mailbox, Message, MessageRef, PageQuery,
    Receipt, Selection, Submit, Template, Transfer,
};

/// One Himalaya session, kept for the whole run so a network client connects once.
pub struct MailClient {
    /// The session; it borrows the config paths and account name for the process's lifetime.
    session: Session<'static>,
}

impl MailClient {
    /// A session over `account` (the default one when `None`) in the files at `config_paths`.
    /// Nothing connects until the first command.
    pub fn new(config_paths: Vec<PathBuf>, account: Option<String>) -> Self {
        let config_paths: &'static [PathBuf] = Vec::leak(config_paths);
        let name = account.map(|name| &*String::leak(name));
        let account = SessionAccount {
            config_paths,
            name,
            backend: Backend::Auto,
        };
        Self {
            session: Session {
                account,
                ..Session::default()
            },
        }
    }

    /// Keeps the connection alive after an idle period (`NOOP` on IMAP).
    pub fn keep_alive(&mut self) -> Result<()> {
        self.session.keep_alive()
    }

    /// The configured account names.
    pub fn accounts(&mut self) -> Result<Vec<String>> {
        let listing: AccountListing = self.run(["account", "list"])?.decode()?;
        Ok(listing
            .accounts
            .into_iter()
            .map(|account| account.name)
            .collect())
    }

    /// The account's mailboxes.
    pub fn list_mailboxes(&mut self) -> Result<Vec<Mailbox>> {
        let listing: MailboxListing = self.run(["mailbox", "list"])?.decode()?;
        Ok(listing.mailboxes)
    }

    /// One page of a mailbox listing or search.
    pub fn list_envelopes(&mut self, query: &PageQuery) -> Result<EnvelopePage> {
        let (page, size) = (query.page.to_string(), query.size.to_string());
        let mailbox = query.mailbox.as_str();
        let mut argv = vec!["envelope"];
        match &query.search {
            Some(search) => argv.extend([
                "search", "-m", mailbox, "-p", &page, "-s", &size, "--", search,
            ]),
            None => argv.extend(["list", "-m", mailbox, "-p", &page, "-s", &size]),
        }
        self.run(argv)?.decode()
    }

    /// Reads a message, marking it seen.
    pub fn read_message(&mut self, message: &MessageRef) -> Result<Message> {
        let argv = [
            "message",
            "read",
            "-m",
            &message.mailbox,
            "--seen",
            "--",
            &message.id,
        ];
        self.run(argv)?.decode()
    }

    /// The raw RFC 5322 source, flags left alone.
    pub fn read_source(&mut self, message: &MessageRef) -> Result<Vec<u8>> {
        let argv = [
            "message",
            "read",
            "-m",
            &message.mailbox,
            "--raw",
            "--",
            &message.id,
        ];
        let printed = self.run(argv)?;
        printed.bytes.context("himalaya printed no message bytes")
    }

    /// Sets or clears `keyword` on the selected messages.
    pub fn set_flag(
        &mut self,
        selection: &Selection,
        keyword: Keyword,
        change: FlagChange,
    ) -> Result<Receipt> {
        let argv = [
            "flag",
            change.verb(),
            "-m",
            &selection.mailbox,
            "-f",
            keyword.name(),
            "--",
        ];
        self.run_on(&argv, &selection.ids)
    }

    /// Moves messages to another mailbox.
    pub fn move_messages(&mut self, transfer: &Transfer) -> Result<Receipt> {
        let argv = [
            "message",
            "move",
            "-f",
            &transfer.from.mailbox,
            "-t",
            &transfer.to,
            "--",
        ];
        self.run_on(&argv, &transfer.from.ids)
    }

    /// Copies messages to another mailbox.
    pub fn copy_messages(&mut self, transfer: &Transfer) -> Result<Receipt> {
        let argv = [
            "message",
            "copy",
            "-f",
            &transfer.from.mailbox,
            "-t",
            &transfer.to,
            "--",
        ];
        self.run_on(&argv, &transfer.from.ids)
    }

    /// Moves messages to the trash, or deletes them for good when already there.
    pub fn delete_messages(&mut self, selection: &Selection) -> Result<Receipt> {
        self.run_on(
            &["message", "delete", "-m", &selection.mailbox, "--"],
            &selection.ids,
        )
    }

    /// Creates a mailbox.
    pub fn create_mailbox(&mut self, name: &str) -> Result<()> {
        self.run(["mailbox", "create", "--", name])?;
        Ok(())
    }

    /// Renames the mailbox with id `mailbox` to `name`.
    pub fn rename_mailbox(&mut self, mailbox: &str, name: &str) -> Result<()> {
        self.run(["mailbox", "rename", "--", mailbox, name])?;
        Ok(())
    }

    /// Deletes a mailbox and everything in it.
    pub fn delete_mailbox(&mut self, mailbox: &str) -> Result<()> {
        self.run(["mailbox", "delete", "--", mailbox])?;
        Ok(())
    }

    /// The template a draft of `kind` starts from.
    pub fn template(&mut self, kind: &ComposeKind) -> Result<Template> {
        match (kind, template_argv(kind)) {
            (ComposeKind::Draft(saved), _) => Ok(Template::from_saved(&self.read_source(saved)?)),
            (_, Some(argv)) => self.run(argv.iter().map(String::as_str))?.decode(),
            (_, None) => Err(anyhow!("no template command for {kind:?}")),
        }
    }

    /// Sends the draft or saves it to the drafts mailbox; a reopened draft is then deleted, as
    /// what was sent or saved replaces it.
    pub fn submit(&mut self, draft: &Draft, submit: Submit) -> Result<Receipt> {
        let body = tempfile::NamedTempFile::new().context("create the body file")?;
        std::fs::write(body.path(), &draft.body).context("write the body file")?;
        let body_path = body
            .path()
            .to_str()
            .context("the temporary path is not UTF-8")?;
        let argv = draft.submit_argv(body_path, submit);
        let receipt = self.run(argv.iter().map(String::as_str))?.decode()?;
        if let ComposeKind::Draft(saved) = &draft.kind {
            let replaced = Selection {
                mailbox: saved.mailbox.clone(),
                ids: vec![saved.id.clone()],
            };
            let _ignored = self.delete_messages(&replaced);
        }
        Ok(receipt)
    }

    /// Writes the message's attachments to `dir`, returning the paths written.
    pub fn download_attachments(
        &mut self,
        message: &MessageRef,
        dir: &Path,
    ) -> Result<Vec<String>> {
        let dir = dir
            .to_str()
            .context("the download directory is not UTF-8")?;
        let argv = [
            "attachment",
            "download",
            "-m",
            &message.mailbox,
            "-d",
            dir,
            "--",
            &message.id,
        ];
        let printed = self.run(argv)?;
        let mut paths = Vec::new();
        collect_paths(printed.value.as_ref(), &mut paths);
        Ok(paths)
    }

    /// Runs `argv` followed by `ids` and decodes its receipt.
    fn run_on(&mut self, argv: &[&str], ids: &[String]) -> Result<Receipt> {
        let argv = argv.iter().copied().chain(ids.iter().map(String::as_str));
        self.run(argv)?.decode()
    }

    /// Runs one command over the session. A partial outcome is a result, its receipt naming the
    /// items that failed.
    fn run<'a>(&mut self, argv: impl IntoIterator<Item = &'a str>) -> Result<Captured> {
        let line = Line::try_parse_from(argv).map_err(|error| anyhow!("{error}"))?;
        let mut printer = Captured::default();
        match line.command.execute(&mut printer, &mut self.session) {
            Ok(()) => Ok(printer),
            Err(error) if partial_outcome_of(&error).is_some() => Ok(printer),
            Err(error) => Err(error),
        }
    }
}

/// A Himalaya command line without the binary name or global flags; the session fixes those.
#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
struct Line {
    /// The command.
    #[command(subcommand)]
    command: Command,
}

/// What a command printed: its JSON value, or the bytes of a raw read.
#[derive(Debug, Default)]
struct Captured {
    /// The value printed under `--json`.
    value: Option<Value>,

    /// The bytes `message read --raw` printed.
    bytes: Option<Vec<u8>>,
}

impl Captured {
    /// The printed value decoded as `T`.
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

/// The `account list` answer.
#[derive(Deserialize)]
struct AccountListing {
    /// The configured accounts.
    accounts: Vec<AccountName>,
}

/// One account of the listing, by name.
#[derive(Deserialize)]
struct AccountName {
    /// The account's name in the config.
    name: String,
}

/// The `mailbox list` answer.
#[derive(Deserialize)]
struct MailboxListing {
    /// The account's mailboxes.
    mailboxes: Vec<Mailbox>,
}

/// Every `path` string anywhere in `value`: the files `attachment download` wrote.
fn collect_paths(value: Option<&Value>, paths: &mut Vec<String>) {
    match value {
        Some(Value::Object(map)) => {
            if let Some(Value::String(path)) = map.get("path") {
                paths.push(path.clone());
            }
            map.values()
                .for_each(|value| collect_paths(Some(value), paths));
        }
        Some(Value::Array(list)) => list
            .iter()
            .for_each(|value| collect_paths(Some(value), paths)),
        Some(Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)) | None => {}
    }
}
