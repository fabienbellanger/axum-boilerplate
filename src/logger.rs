//! Logger module for customize `Tracing` logs

use crate::errors::{CliError, CliResult};
use tracing_subscriber::{fmt::format::JsonFields, prelude::*, EnvFilter, Registry};

// Examples:
// - https://github.com/gsson/mini-web-rs
// - https://github.com/shanesveller/axum-rest-example

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init(environment: &str, path: &str, filename: &str) -> CliResult<()> {
    let (is_production, filter) = match environment {
        "production" => (
            true,
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("error")),
        ),
        "test" => (false, EnvFilter::new("error")),
        _ => (
            false,
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        ),
    };

    let format = tracing_subscriber::fmt::format()
        .with_level(true) // don't include levels in formatted output
        .with_target(true) // don't include targets
        .with_thread_ids(true) // include the thread ID of the current thread
        .with_thread_names(true) // include the name of the current thread
        .with_file(true)
        .with_line_number(true);

    if is_production {
        let file_appender = tracing_appender::rolling::daily(path, filename);
        let layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .event_format(format.json())
            .fmt_fields(JsonFields::new())
            .with_writer(file_appender);

        let subscriber = Registry::default().with(filter).with(layer);

        tracing::subscriber::set_global_default(subscriber).map_err(|err| CliError::ConfigError(err.to_string()))?;
    } else {
        let layer = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .event_format(format.pretty())
            .with_writer(std::io::stdout);

        let subscriber = Registry::default().with(filter).with(layer);

        tracing::subscriber::set_global_default(subscriber).map_err(|err| CliError::ConfigError(err.to_string()))?;
    }

    Ok(())
}
