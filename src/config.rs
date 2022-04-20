//! Configuration module

use color_eyre::Result;
use serde::Deserialize;

/// Represents configuration structure
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub environment: String,
    pub rust_log: String,
    pub logs_path: String,
    pub logs_file: String,
    pub server_url: String,
    pub server_port: String,
    pub jwt_secret_key: String,
    pub jwt_lifetime: i64,
    pub database_url: String,
    pub database_auto_migration: bool,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_max_lifetime: u64,
    pub database_connect_timeout: u64,
    pub database_idle_timeout: u64,
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
