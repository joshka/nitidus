use std::{panic, path::PathBuf};

use clap::Parser;
use mail_client::MailClient;

mod app;
mod mail_client;
mod tui;

#[derive(Debug, Default, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A path to a himalaya configuration file.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// The name of the account to use from the configuration file.
    #[arg(short, long, value_name = "ACCOUNT", alias = "account")]
    account_name: Option<String>,

    /// The mail folder to open.
    #[arg(short, long, value_name = "FOLDER")]
    folder: Option<String>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    install_hooks()?;

    let args = Args::parse();
    let mail_client = MailClient::init(&args).await?;
    let mut terminal = &mut tui::init()?;
    app::run(&mut terminal, mail_client).await?;
    tui::restore()?;
    Ok(())
}

/// This replaces the standard color_eyre panic and error hooks with hooks that
/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();

    // convert from a color_eyre PanicHook to a standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        tui::restore().unwrap();
        panic_hook(panic_info);
    }));

    // convert from a color_eyre EyreHook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            tui::restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}
