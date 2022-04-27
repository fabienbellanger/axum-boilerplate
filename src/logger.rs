//! Logger module for customize `Tracing` logs

use tracing_subscriber::{fmt::format::JsonFields, prelude::*, EnvFilter, Registry};

// TODO: try:
// - https://github.com/gsson/mini-web-rs
// - https://github.com/shanesveller/axum-rest-example

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
// Read: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html
pub fn init(environment: &str, path: &str, filename: &str) {
    if environment == "production" {
        // Production environment
        let prod_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("error"));
        let prod_format = tracing_subscriber::fmt::format()
            .with_level(true) // don't include levels in formatted output
            .with_target(true) // don't include targets
            .with_thread_ids(true) // include the thread ID of the current thread
            .with_thread_names(true) // include the name of the current thread
            .with_file(true)
            .with_line_number(true)
            .json();
        let file_appender = tracing_appender::rolling::daily(path, filename);
        let prod_layer = tracing_subscriber::fmt::layer()
            .event_format(prod_format)
            .with_ansi(false)
            .fmt_fields(JsonFields::new())
            .with_writer(file_appender);
        let prod_subscriber = Registry::default().with(prod_filter).with(prod_layer);

        tracing::subscriber::set_global_default(prod_subscriber).expect("unable to set global production subscriber");
    } else {
        // Development environment
        let dev_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let dev_format = tracing_subscriber::fmt::format()
            .with_level(true) // don't include levels in formatted output
            .with_target(true) // don't include targets
            .with_thread_ids(true) // include the thread ID of the current thread
            .with_thread_names(true) // include the name of the current thread
            .with_file(true)
            .with_line_number(true)
            .pretty();
        let dev_layer = tracing_subscriber::fmt::layer()
            .event_format(dev_format)
            .with_ansi(true)
            .with_writer(std::io::stdout);
        let dev_subscriber = Registry::default().with(dev_filter).with(dev_layer);

        tracing::subscriber::set_global_default(dev_subscriber).expect("unable to set global development subscriber");
    }
}
