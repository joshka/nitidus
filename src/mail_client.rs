use std::{path::PathBuf, sync::Arc};

use color_eyre::eyre::{bail, eyre};
use email::{
    account::config::DEFAULT_PAGE_SIZE,
    backend::{Backend, BackendBuilder},
    email::{envelope::Envelopes, message::Messages},
    envelope::{list::ListEnvelopes, Id},
    folder::INBOX,
    imap::{ImapContextBuilder, ImapContextSync},
    message::get::GetMessages,
};
use himalaya::config::TomlConfig;

use crate::config;

pub struct MailClient {
    folder: Option<String>,
    backend: Backend<ImapContextSync>,
}

impl MailClient {
    pub async fn init() -> color_eyre::Result<Self> {
        let app_config = config::get();
        let himalaya_config = load_config(app_config.himalaya_config.clone()).await?;
        let account_name = app_config.account_name.as_ref().map(String::as_ref);
        let (toml_account_config, account_config) = himalaya_config
            .into_account_configs(account_name, true)
            .map_err(|err| {
                eyre!(
                    "cannot find account `{}` in config file: {}",
                    account_name.unwrap_or("default"),
                    err
                )
            })?;

        let Some(ref imap_config) = toml_account_config.imap else {
            bail!("missing imap config")
        };
        let imap_config = Arc::new(imap_config.clone());
        let imap_ctx = ImapContextBuilder::new(account_config.clone(), imap_config);
        let backend = BackendBuilder::new(account_config.clone(), imap_ctx.clone())
            .build::<Backend<ImapContextSync>>()
            .await
            .map_err(|err| eyre!("cannot create imap backend: {}", err))?;

        Ok(Self {
            folder: app_config.folder.clone(),
            backend,
        })
    }

    pub fn folder_or_default(&self) -> &str {
        self.folder.as_ref().map_or(INBOX, String::as_ref)
    }

    pub async fn load_folder(&self) -> color_eyre::Result<Envelopes> {
        let page = 0;
        let envelopes = self
            .backend
            .list_envelopes(self.folder_or_default(), DEFAULT_PAGE_SIZE, page)
            .await
            .map_err(|err| eyre!("cannot list envelopes: {}", err))?;
        Ok(envelopes)
    }

    pub async fn load_messages(&self, id: &str) -> color_eyre::Result<Messages> {
        let id = Id::single(id);
        let emails = self
            .backend
            .get_messages(self.folder_or_default(), &id)
            .await
            .map_err(|err| eyre!("cannot get messages: {}", err))?;
        Ok(emails)
    }
}

async fn load_config(path: Option<PathBuf>) -> color_eyre::Result<TomlConfig> {
    let path =
        path.ok_or_else(|| eyre!("config file not found, please run `himalaya` to create one"))?;

    let config = TomlConfig::from_some_path_or_default(Some(path))
        .await
        .map_err(|err| {
            eyre!(
                "cannot load config file: {} (hint: run `himalaya` to create one)",
                err,
            )
        })?;
    Ok(config)
}
