//! Configuration module

use color_eyre::Result;
use serde::Deserialize;

/// Represents configuration structure
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    /// Environment: `developement` or `production`
    pub environment: String,

    /// Logs used by Axum, sqlx, etc.
    pub rust_log: String,

    /// Path of log files
    pub logs_path: String,
    /// Log file name
    pub logs_file: String,

    /// Server URL
    pub server_url: String,
    /// Server port
    pub server_port: String,
    /// Server requests timeout
    pub request_timeout: u64,

    /// JWT secret key
    pub jwt_secret_key: String,
    /// JWT lifetime
    pub jwt_lifetime: i64,

    /// CORS Allow Origin Headers
    pub cors_allow_origin: String,

    pub database_url: String,
    pub database_auto_migration: bool,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_max_lifetime: u64,
    pub database_connect_timeout: u64,
    pub database_idle_timeout: u64,

    pub redis_url: String,
    pub redis_prefix: String,
    pub redis_connection_timeout: u64,

    pub limiter_enabled: bool,
    pub limiter_requests_by_second: i64,
    pub limiter_expire_in_seconds: i64,
    pub limiter_white_list: String,
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
