use std::{path::PathBuf, sync::Arc};

use color_eyre::eyre::{bail, eyre, Context, OptionExt};
use email::{
    account::config::DEFAULT_PAGE_SIZE,
    backend::{Backend, BackendBuilder},
    config::Config,
    email::{envelope::Envelopes, message::Messages},
    envelope::{
        list::{ListEnvelopes, ListEnvelopesOptions},
        Id,
    },
    folder::INBOX,
    imap::{ImapContext, ImapContextBuilder},
    message::get::GetMessages,
};
use himalaya::config::TomlConfig;
use pimalaya_tui::terminal::config::TomlConfig as _;
use tracing::{info, instrument};

use crate::config;

pub struct MailClient {
    folder: Option<String>,
    backend: Backend<ImapContext>,
}

impl MailClient {
    #[instrument]
    pub async fn init() -> color_eyre::Result<Self> {
        info!("initializing mail client");
        secret::keyring::set_global_service_name("himalaya-cli");

        let app_config = config::get();
        let himalaya_config = load_config(app_config.himalaya_config.clone()).await?;
        let account_name = app_config.account_name.as_ref().map(String::as_ref);
        let (toml_account_config, account_config) = himalaya_config
            .into_account_configs(account_name, |c: &Config, name| c.account(name).ok())
            .map_err(|err| {
                eyre!(
                    "cannot find account `{}` in config file: {err}",
                    account_name.unwrap_or("default"),
                )
            })?;

        let Some(imap_config) = toml_account_config.imap_config() else {
            bail!("missing backend config")
        };

        info!(?account_config, ?imap_config, "creating imap backend");

        let account_config = Arc::new(account_config.clone());
        let imap_config = Arc::new(imap_config.clone());
        let imap_ctx = ImapContextBuilder::new(account_config.clone(), imap_config);
        let backend = BackendBuilder::new(account_config, imap_ctx.clone())
            .build()
            .await
            .wrap_err("cannot create imap backend")?;

        Ok(Self {
            folder: app_config.folder.clone(),
            backend,
        })
    }

    pub fn folder_or_default(&self) -> &str {
        self.folder.as_ref().map_or(INBOX, String::as_ref)
    }

    #[instrument(skip(self))]
    pub async fn load_folder(&self) -> color_eyre::Result<Envelopes> {
        let folder = self.folder_or_default();
        info!("loading folder {folder}");
        let options = ListEnvelopesOptions {
            page_size: DEFAULT_PAGE_SIZE,
            page: 0,
            query: None,
        };
        let envelopes = self
            .backend
            .list_envelopes(folder, options)
            .await
            .wrap_err("cannot list envelopes")?;
        Ok(envelopes)
    }

    #[instrument(skip(self))]
    pub async fn load_messages(&self, id: &str) -> color_eyre::Result<Messages> {
        let id = Id::single(id);
        let folder = self.folder_or_default();
        info!("loading messages for {id} in folder {folder}");
        let emails = self
            .backend
            .get_messages(folder, &id)
            .await
            .wrap_err("cannot load messages")?;
        Ok(emails)
    }
}

#[instrument]
async fn load_config(path: Option<PathBuf>) -> color_eyre::Result<TomlConfig> {
    info!("loading himalaya config");
    let path = path.ok_or_eyre("config file not found, please run `himalaya` to create one")?;

    let config = TomlConfig::from_paths_or_default(&[path])
        .await
        .wrap_err("cannot load config file. (hint: run `himalaya` to create one)")?;
    Ok(config)
}
