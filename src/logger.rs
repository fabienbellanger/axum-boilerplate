//! Logger module for customize `Tracing` logs

use tracing_subscriber::{prelude::*, EnvFilter, Registry};

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
// Read: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html
pub fn init(environment: &str, path: &str, filename: &str) {
    // Production environment
    let prod_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("eeror"));
    let prod_format = tracing_subscriber::fmt::format()
        .with_level(true) // don't include levels in formatted output
        .with_target(true) // don't include targets
        .with_thread_ids(false) // include the thread ID of the current thread
        .with_thread_names(false) // include the name of the current thread
        .with_file(true)
        .with_line_number(true)
        .json();
    let file_appender = tracing_appender::rolling::daily(path, filename);
    let prod_layer = tracing_subscriber::fmt::layer()
        .event_format(prod_format)
        .with_writer(file_appender);
    let prod_subscriber = Registry::default().with(prod_filter).with(prod_layer);

    // Development environment
    let dev_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let dev_format = tracing_subscriber::fmt::format()
        .with_level(true) // don't include levels in formatted output
        .with_target(false) // don't include targets
        .with_thread_ids(false) // include the thread ID of the current thread
        .with_thread_names(false) // include the name of the current thread
        .with_file(true)
        .with_line_number(true)
        .compact();
    let dev_layer = tracing_subscriber::fmt::layer()
        .event_format(dev_format)
        .with_writer(std::io::stdout);
    let dev_subscriber = Registry::default().with(dev_filter).with(dev_layer);

    if environment == "production" {
        tracing::subscriber::set_global_default(prod_subscriber).expect("unable to set global production subscriber");
    } else {
        tracing::subscriber::set_global_default(dev_subscriber).expect("unable to set global development subscriber");
    }
}
