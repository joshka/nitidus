use std::{fs, path::PathBuf};

use color_eyre::eyre::{bail, eyre, WrapErr};
use email::{
    account::{DEFAULT_INBOX_FOLDER, DEFAULT_PAGE_SIZE},
    backend::BackendBuilder,
    email::{Envelopes, Messages},
};
use himalaya::config::DeserializedConfig;

use crate::Args;

#[derive(Debug, Default)]
pub struct MailClient {
    config: DeserializedConfig,
    // account_config: AccountConfig,
    backend_builder: BackendBuilder,
    folder: Option<String>,
}

impl MailClient {
    pub async fn init(args: &Args) -> color_eyre::Result<Self> {
        let config = load_config(args.config.clone())?;
        let account_name = args.account_name.as_ref().map(String::as_ref);
        let account_config = config.to_account_config(account_name).map_err(|err| {
            eyre!(
                "cannot find account `{}` in config file: {}",
                account_name.unwrap_or("default"),
                err
            )
        })?;
        let backend_builder = BackendBuilder::new(account_config.clone());
        Ok(Self {
            config,
            // account_config,
            backend_builder,
            folder: args.folder.clone(),
        })
    }

    pub fn folder_or_default(&self) -> &str {
        self.folder
            .as_ref()
            .map_or(DEFAULT_INBOX_FOLDER, String::as_ref)
    }

    pub async fn load_folder(&self) -> color_eyre::Result<Envelopes> {
        let mut backend = self.backend_builder.clone().into_build().await?;
        let page_size = self
            .config
            .email_listing_page_size
            .unwrap_or(DEFAULT_PAGE_SIZE);
        let page = 0;
        let envelopes = backend
            .list_envelopes(self.folder_or_default(), page_size, page)
            .await?;
        Ok(envelopes)
    }

    pub async fn load_messages(&self, id: &str) -> color_eyre::Result<Messages> {
        let mut backend = self.backend_builder.clone().into_build().await?;
        let emails = backend
            .get_emails(self.folder_or_default(), vec![id])
            .await?;
        Ok(emails)
    }
}

fn load_config(path: Option<PathBuf>) -> color_eyre::Result<DeserializedConfig> {
    let path = path
        .or_else(DeserializedConfig::path)
        .ok_or_else(|| eyre!("config file not found, please run `himalaya` to create one"))?;
    let content = fs::read_to_string(&path).wrap_err("cannot read config file")?;
    let config: DeserializedConfig =
        toml::from_str(&content).wrap_err("cannot parse config file")?;
    if config.accounts.is_empty() {
        bail!("no accounts found in config file, please run `himalaya` to add one")
    }
    Ok(config)
}
