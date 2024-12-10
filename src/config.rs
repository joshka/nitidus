use std::{path::PathBuf, process::exit, sync::OnceLock};

use clap::{Parser, Subcommand};
use color_eyre::eyre::{eyre, OptionExt};
use directories::ProjectDirs;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use himalaya::config::TomlConfig;
use pimalaya_tui::terminal::config::TomlConfig as _;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr, NoneAsEmptyString};
use tracing::level_filters::LevelFilter;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Command line arguments.
///
/// Implements Serialize so that we can use it as a source for Figment configuration.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Parser, Serialize, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The directory to use for storing application data.
    #[arg(long, value_name = "DIR")]
    data_dir: Option<PathBuf>,

    /// A path to a nitidus configuration file.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// A path to the himalaya configuration file.
    ///
    /// If not specified, the default himalaya configuration file will be used.
    #[arg(long, value_name = "FILE")]
    himalaya_config: Option<PathBuf>,

    /// The log level to use.
    ///
    /// Valid values are: error, warn, info, debug, trace, off. The default is info.
    #[arg(long, value_name = "LEVEL", default_value = "info", alias = "log")]
    #[serde_as(as = "NoneAsEmptyString")]
    log_level: Option<LevelFilter>,

    /// The name of the account to use from the configuration file.
    #[arg(short, long, value_name = "ACCOUNT", alias = "account")]
    #[serde_as(as = "NoneAsEmptyString")]
    account_name: Option<String>,

    /// The mail folder to open.
    #[arg(short, long, value_name = "FOLDER")]
    #[serde_as(as = "NoneAsEmptyString")]
    folder: Option<String>,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

/// Subcommands.
#[derive(Debug, Serialize, Subcommand, Clone, Copy)]
pub enum Command {
    /// Print the default configuration file.
    #[serde(rename = "print-default-config")]
    PrintDefaultConfig,
}

/// Application configuration.
///
/// This is the main configuration struct for the application.
#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    /// The directory to use for storing application data (logs etc.).
    pub data_dir: PathBuf,

    /// A path to a himalaya configuration file.
    ///
    /// If not specified, the default himalaya configuration file will be used.
    /// TODO: use the nitidus config file as the config file for himalaya
    pub himalaya_config: Option<PathBuf>,

    /// The log level to use. Valid values are: error, warn, info, debug, trace, off. The default is
    /// info.
    #[serde_as(as = "DisplayFromStr")]
    pub log_level: LevelFilter,

    /// The name of the account to use from the configuration file.
    ///
    /// If not specified, the default account will be used.
    pub account_name: Option<String>,

    /// The mail folder to open.
    ///
    /// If not specified, the default inbox folder will be used.
    pub folder: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = default_data_dir();
        let himalaya_config = default_himalaya_config();
        Self {
            data_dir,
            himalaya_config,
            log_level: LevelFilter::INFO,
            account_name: None,
            folder: None,
        }
    }
}

/// Returns the directory to use for storing data files.
fn default_data_dir() -> PathBuf {
    project_dirs()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap()
}

/// Returns the path to the default configuration file.
fn default_config_file() -> PathBuf {
    project_dirs()
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .unwrap()
}

/// Returns the project directories.
fn project_dirs() -> color_eyre::Result<ProjectDirs> {
    ProjectDirs::from("net", "joshka", "nitidus").ok_or_eyre("user home directory not found")
}

/// Returns the path to the default himalaya configuration file.
fn default_himalaya_config() -> Option<PathBuf> {
    TomlConfig::default_path().ok()
}

/// Initialize the application configuration.
///
/// This function should be called before any other function in the application.
/// It will initialize the application config from the following sources:
/// - default values
/// - a configuration file
/// - environment variables
/// - command line arguments
pub fn init() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    // TODO use project_dirs config file for the default config file
    let config_file = cli.config.clone().unwrap_or_else(default_config_file);
    let config = Figment::new()
        .merge(Serialized::defaults(AppConfig::default()))
        .merge(Toml::file(config_file))
        .merge(Env::prefixed("NITIDUS_"))
        .merge(Serialized::defaults(cli.clone()))
        .extract::<AppConfig>()?;

    match cli.command {
        Some(Command::PrintDefaultConfig) => {
            println!("{:#?}", config);
            exit(0);
        }
        _ => (),
    }

    CONFIG
        .set(config)
        .map_err(|config| eyre!("failed to set config {config:?}"))
}

/// Get the application configuration.
///
/// This function should only be called after [`init()`] has been called.
///
/// # Panics
///
/// This function will panic if [`init()`] has not been called.
pub fn get() -> &'static AppConfig {
    CONFIG.get().expect("config not initialized")
}
