//! Configuration module

use color_eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;

/// Represents configuration structure
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub server_url: String,
    pub server_port: String,
    pub rust_log: String,
}

impl Config {
    /// from_env loads configuration from environment variables
    pub fn from_env() -> Result<Config> {
        dotenv::dotenv().ok();

        let mut c = config::Config::new();
        c.merge(config::Environment::default())?;

        c.try_into().context("loading configuration from environment")
    }
}
