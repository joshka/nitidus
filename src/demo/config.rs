//! The Himalaya config naming the demo accounts.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::ME;
use super::maildir::Accounts;

/// The work account's address.
const WORK: (&str, &str) = ("Rae Field", "rae@work.test");

/// Writes the config for the Maildir accounts, sending through the sink on `port`.
pub fn write_maildir(dir: &Path, accounts: &Accounts, port: u16) -> Result<PathBuf> {
    let tables: Vec<String> = accounts
        .named()
        .iter()
        .map(|&(name, root)| maildir_account(name, root, port))
        .collect();
    write(dir, &tables.join("\n"))
}

/// Writes the config for the pimdir store at `root`.
pub fn write_pimdir(dir: &Path, root: &Path) -> Result<PathBuf> {
    let mut lines = identity("demo", ME);
    lines.push(format!("pimdir.root = {:?}", root.display().to_string()));
    for (role, mailbox) in [
        ("inbox", "INBOX"),
        ("sent", "Sent"),
        ("drafts", "Drafts"),
        ("trash", "Trash"),
    ] {
        lines.push(format!("mailbox.alias.{role} = \"imap/{mailbox}\""));
    }
    write(dir, &lines.join("\n"))
}

/// One Maildir account's table.
fn maildir_account(name: &str, root: &Path, port: u16) -> String {
    let mut lines = identity(name, if name == "demo" { ME } else { WORK });
    lines.push("signature = \"Rae\"".to_owned());
    lines.push(format!("maildir.root = {:?}", root.display().to_string()));
    lines.push(format!("smtp.server = \"smtp://127.0.0.1:{port}\""));
    for (role, mailbox) in [("sent", "Sent"), ("drafts", "Drafts"), ("trash", "Trash")] {
        lines.push(format!("mailbox.alias.{role} = \"{mailbox}\""));
    }
    lines.push("message.send.save-copy = true\n".to_owned());
    lines.join("\n")
}

/// An account's table header and identity lines; the `demo` account is the default.
fn identity(name: &str, (display, email): (&str, &str)) -> Vec<String> {
    let mut lines = vec![format!("[accounts.{name}]")];
    if name == "demo" {
        lines.push("default = true".to_owned());
    }
    lines.push(format!("email = \"{email}\""));
    lines.push(format!("display-name = \"{display}\""));
    lines
}

/// Writes `toml` as `dir/config.toml`.
fn write(dir: &Path, toml: &str) -> Result<PathBuf> {
    let path = dir.join("config.toml");
    fs::write(&path, format!("{toml}\n")).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}
