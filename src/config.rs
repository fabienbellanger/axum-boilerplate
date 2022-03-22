//! Configuration module

use color_eyre::Result;
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

        Ok(config::Config::builder()
            .add_source(config::Environment::default())
            .build()?
            .try_deserialize()?)
    }
}
