//! The command line: which account to open, the theme, and the demo.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::ui::theme::{self, Theme};

/// A terminal mail client on top of Himalaya.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// A Himalaya configuration file (repeatable); Himalaya's default otherwise.
    #[arg(short, long = "config", value_name = "FILE")]
    pub config: Vec<PathBuf>,

    /// The account to open, the default one otherwise.
    #[arg(short, long)]
    pub account: Option<String>,

    /// The mailbox to open first.
    #[arg(short, long)]
    pub mailbox: Option<String>,

    /// Color theme; `auto` reads $COLORFGBG and falls back to dark.
    #[arg(long, value_enum, default_value = "auto")]
    pub theme: ThemeChoice,

    /// What to run instead of the configured account.
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

/// The subcommands.
#[derive(Debug, Subcommand)]
pub enum CliCommand {
    /// Open fake accounts, no login needed.
    Demo(DemoArgs),
}

/// How the demo runs.
#[derive(Debug, Args)]
pub struct DemoArgs {
    /// Where to write the fake accounts; a temporary directory otherwise.
    #[arg(long)]
    pub dir: Option<PathBuf>,

    /// Seed a pimdir store instead of Maildir (writes stay queued: no sync engine runs).
    #[arg(long)]
    pub pimdir: bool,

    /// Write the accounts and print the config path without starting the UI, to point another
    /// client at the same mail.
    #[arg(long, requires = "dir")]
    pub setup_only: bool,
}

/// The theme the user asked for.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ThemeChoice {
    /// Light when `$COLORFGBG` says the terminal is light, dark otherwise.
    Auto,

    /// The dark palette.
    Dark,

    /// The light palette.
    Light,
}

impl ThemeChoice {
    /// The palette this choice resolves to.
    pub fn resolve(self) -> Theme {
        match self {
            Self::Light => theme::LIGHT,
            Self::Auto if terminal_is_light() => theme::LIGHT,
            Self::Dark | Self::Auto => theme::DARK,
        }
    }
}

/// Whether `$COLORFGBG` ("fg;bg") names a light background: color 7, or 9 to 15.
fn terminal_is_light() -> bool {
    let value = std::env::var("COLORFGBG").unwrap_or_default();
    let background = value
        .rsplit(';')
        .next()
        .and_then(|bg| bg.parse::<u8>().ok());
    background.is_some_and(|bg| bg == 7 || bg >= 9)
}
