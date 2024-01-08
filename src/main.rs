use mail_client::MailClient;
use tracing::info;

mod app;
mod config;
mod errors;
mod logging;
mod mail_client;
mod tui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    config::init()?;
    let _guard = logging::init()?;
    errors::install_hooks()?;

    info!("starting nitidus");
    let terminal = tui::init()?;
    let mail_client = MailClient::init().await?;
    app::run(terminal, mail_client).await?;
    tui::restore()?;
    Ok(())
}
