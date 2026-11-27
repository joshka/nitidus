//! Fake accounts, so nitidus can be tried and recorded without logging in anywhere.
//!
//! Two Maildir accounts (or one pimdir store) filled from [`messages`], and an SMTP server on
//! localhost that accepts everything ([`smtp`]); mail to the demo address comes back to its inbox.
//! The demo runs the same code paths as a real account.

mod config;
mod maildir;
mod messages;
mod pimdir;
mod smtp;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use tempfile::TempDir;

use crate::worker::AccountSpec;

/// The demo address, which the SMTP sink delivers back to the inbox.
const ME: (&str, &str) = ("Rae Field", "rae@nitidus.test");

/// Where the demo's mail lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Store {
    /// Two Maildir accounts, sending through the local SMTP sink.
    Maildir,

    /// One pimdir store; writes queue for a sync engine that never runs.
    Pimdir,
}

/// Demo accounts written to disk.
pub struct Demo {
    /// The temporary directory holding them, removed when the demo ends; `None` for a directory
    /// the user named.
    _temporary: Option<TempDir>,

    /// The Himalaya config naming the accounts.
    pub config: PathBuf,
}

impl Demo {
    /// Writes the demo accounts to `dir`, or to a temporary directory when `None`.
    pub fn create(dir: Option<PathBuf>, store: Store) -> Result<Self> {
        let temporary = match dir {
            Some(_) => None,
            None => Some(TempDir::new().context("create a temporary directory")?),
        };
        let root = dir
            .or_else(|| temporary.as_ref().map(|t| t.path().to_owned()))
            .unwrap_or_default();
        let config = create(&root, store)?;
        Ok(Self {
            _temporary: temporary,
            config,
        })
    }

    /// The demo's default account.
    pub fn account(&self) -> AccountSpec {
        AccountSpec {
            config: vec![self.config.clone()],
            account: Some("demo".into()),
        }
    }
}

/// Writes the demo accounts under `dir`, returning the config path. The Maildir store also starts
/// the SMTP sink, which runs until the process ends.
pub fn create(dir: &Path, store: Store) -> Result<PathBuf> {
    create_at(dir, store, Local::now())
}

/// As [`create`], with the mail dated relative to `now` rather than the clock, so what a test
/// renders doesn't change with the time of day.
pub fn create_at(dir: &Path, store: Store, now: DateTime<Local>) -> Result<PathBuf> {
    let accounts = maildir::write_accounts(dir, now)?;
    match store {
        Store::Maildir => {
            let port = smtp::start(accounts.personal().join("new"))?;
            config::write_maildir(dir, &accounts, port)
        }
        Store::Pimdir => {
            let root = pimdir::seed(dir, accounts.personal())?;
            config::write_pimdir(dir, &root)
        }
    }
}
