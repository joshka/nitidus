use tracing::debug;
use tracing_appender::non_blocking::WorkerGuard;

use crate::config;

const LOG_FILE_NAME: &str = "nitidus.log";

/// Initialize logging.
///
/// Sets up logging for the application with the following configuration:
/// - log level is pulled from the app config
/// - logs are written to a file that is rotated hourly and stored in the app's data directory
/// - logs are written to a tui logger
/// - spantraces are captured (for color-eyre)
///
/// Returns a [`WorkerGuard`][guard] which is returned by [`tracing_appender::non_blocking`][non_blocking]
/// to ensure buffered logs are flushed to their output in the case of abrupt terminations of a process.
pub fn init() -> color_eyre::Result<WorkerGuard> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let app_config = config::get();

    // log to a file that is rotated hourly and stored in the app's data directory
    let log_file = tracing_appender::rolling::hourly(app_config.data_dir.clone(), LOG_FILE_NAME);
    let (log_file_writer, worker_guard) = tracing_appender::non_blocking(log_file);
    let log_file_layer = fmt::layer()
        .with_writer(log_file_writer)
        .with_ansi(false)
        .compact();

    tracing_subscriber::registry()
        .with(ErrorLayer::default()) // capture spantraces
        .with(EnvFilter::from_default_env().add_directive(app_config.log_level.into()))
        .with(log_file_layer)
        .with(tui_logger::tracing_subscriber_layer())
        .try_init()?;

    debug!("logging initialized");
    Ok(worker_guard)
}
