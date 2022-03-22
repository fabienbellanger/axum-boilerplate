//! Logger module for customize logs

// TODO: Exit from boilerplate

mod formatter_layer;
mod storage_layer;

use formatter_layer::CustomFormattingLayer;
use storage_layer::JsonStorageLayer;
use tracing::Subscriber;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

pub fn get_subscriber(
    level: String,
    sink: impl for<'a> MakeWriter<'a> + Send + Sync + 'static,
) -> impl Subscriber + Sync + Send {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let formatting_layer = CustomFormattingLayer::new(sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
