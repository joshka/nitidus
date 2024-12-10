use mail_client::MailClient;
use tracing::info;

mod app;
mod config;
mod logging;
mod mail_client;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    config::init()?;
    let _guard = logging::init()?;

    info!("starting nitidus");
    let mail_client = MailClient::init().await?;
    let terminal = ratatui::init();
    let result = app::run(terminal, mail_client).await;
    ratatui::restore();
    result
}
