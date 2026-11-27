//! nitidus: a TUI email client on top of Himalaya.

mod app;
mod compose;
mod demo;
mod mail;
mod ui;
mod worker;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// A Himalaya configuration file (repeatable); Himalaya's default otherwise.
    #[arg(short, long = "config", value_name = "FILE")]
    config: Vec<PathBuf>,
    /// The account to open, the default one otherwise.
    #[arg(short, long)]
    account: Option<String>,
    /// The mailbox to open first.
    #[arg(short, long)]
    mailbox: Option<String>,
    /// Colour theme; `auto` reads $COLORFGBG and falls back to dark.
    #[arg(long, value_enum, default_value = "auto")]
    theme: ThemeArg,
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum ThemeArg {
    Auto,
    Dark,
    Light,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Open a fake Maildir account, no login needed.
    Demo {
        /// Where to write the fake account; a temporary directory otherwise.
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Seed a pimdir store instead of Maildir (writes stay queued: no
        /// sync engine runs).
        #[arg(long)]
        pimdir: bool,
        /// Write the account and print its config path, without starting
        /// the UI, to point another client at the same mail.
        #[arg(long, requires = "dir")]
        setup_only: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let _tmp;
    let (config, account) = match cli.command {
        Some(Cmd::Demo {
            dir,
            pimdir,
            setup_only,
        }) => {
            let dir = if let Some(dir) = dir {
                dir
            } else {
                _tmp = tempfile::tempdir()?;
                _tmp.path().to_path_buf()
            };
            let config = demo::create(&dir, pimdir)?;
            if setup_only {
                println!("{}", config.display());
                return Ok(());
            }
            (vec![config], Some("demo".to_string()))
        }
        None => (cli.config, cli.account),
    };
    let light = match cli.theme {
        ThemeArg::Light => true,
        ThemeArg::Dark => false,
        // "fg;bg": a background of 7 or 9-15 is a light terminal.
        ThemeArg::Auto => std::env::var("COLORFGBG")
            .ok()
            .and_then(|v| v.rsplit(';').next()?.parse::<u8>().ok())
            .is_some_and(|bg| bg == 7 || bg >= 9),
    };
    ui::theme::set(if light {
        ui::theme::LIGHT
    } else {
        ui::theme::DARK
    });
    let app = app::App::new(config, account, cli.mailbox);
    // OSC 11 paints the whole terminal background (padding and any
    // unpainted cells) in the theme's base colour; OSC 111 restores it.
    let ratatui::style::Color::Rgb(r, g, b) = ui::theme().bg else {
        unreachable!()
    };
    set_terminal_bg(&format!("\x1b]11;#{r:02x}{g:02x}{b:02x}\x1b\\"));
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        set_terminal_bg(RESET_BG);
        hook(info);
    }));
    let result = ratatui::run(|terminal| app.run(terminal));
    set_terminal_bg(RESET_BG);
    result
}

const RESET_BG: &str = "\x1b]111\x1b\\";

fn set_terminal_bg(seq: &str) {
    use std::io::Write;
    let mut out = std::io::stdout();
    let _ = out.write_all(seq.as_bytes()).and_then(|()| out.flush());
}
