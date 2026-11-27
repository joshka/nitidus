//! nitidus: a terminal mail client on top of Himalaya.
//!
//! The pieces, in the order a key press travels:
//!
//! - [`cli`] picks the account and theme, or [`demo`] builds fake accounts.
//! - [`app`] owns the screen state, maps keys to actions through its keymap, and turns actions into
//!   requests.
//! - [`worker`] runs each request on its own thread over one [`mail`] session, which runs
//!   Himalaya's commands in-process and decodes their JSON.
//! - [`ui`] holds the components, each owning its state and drawing itself.
//! - [`system`] and [`terminal`] are the edges: the editor, the opener, and the terminal itself.

mod app;
mod cli;
mod demo;
mod mail;
mod system;
mod terminal;
mod ui;
mod worker;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, CliCommand, DemoArgs};
use crate::demo::{Demo, Store};
use crate::worker::AccountSpec;

fn main() -> Result<()> {
    let cli = Cli::parse();
    ui::theme::set(cli.theme.resolve());
    match cli.command {
        Some(CliCommand::Demo(args)) => run_demo(&args, cli.mailbox),
        None => run(
            AccountSpec {
                config: cli.config,
                account: cli.account,
            },
            cli.mailbox,
        ),
    }
}

/// Builds the demo accounts and runs on them, or prints their config path for another client.
fn run_demo(args: &DemoArgs, mailbox: Option<String>) -> Result<()> {
    let store = if args.pimdir {
        Store::Pimdir
    } else {
        Store::Maildir
    };
    let demo = Demo::create(args.dir.clone(), store)?;
    if args.setup_only {
        print_config_path(&demo);
        return Ok(());
    }
    run(demo.account(), mailbox)
}

/// Prints the demo's config path, the only thing nitidus ever prints.
#[expect(
    clippy::print_stdout,
    reason = "--setup-only answers on stdout for scripts to read"
)]
fn print_config_path(demo: &Demo) {
    println!("{}", demo.config.display());
}

/// Runs the app on `account`, opening `mailbox` first when given.
fn run(account: AccountSpec, mailbox: Option<String>) -> Result<()> {
    let app = app::App::new(account, mailbox);
    terminal::run(ui::theme().bg, |terminal| app.run(terminal))
}
