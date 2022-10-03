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
    /// Server requests timeout (in second)
    pub request_timeout: u64,

    /// JWT secret key
    pub jwt_secret_key: String,
    /// JWT lifetime
    pub jwt_lifetime: i64,

    /// CORS Allow Origin Headers (URLs delimited by a comma)
    pub cors_allow_origin: String,

    /// Database URL (Ex.: mysql://root:root@127.0.0.1:3306/axum)
    pub database_url: String,
    /// Database auto migration enabled
    pub database_auto_migration: bool,
    /// Database maximum connections (in second)
    pub database_max_connections: u32,
    /// Database minimum connections (in second)
    pub database_min_connections: u32,
    /// Database maximum lifetime (in second)
    pub database_max_lifetime: u64,
    /// Database connection timeout (in second)
    pub database_connect_timeout: u64,
    /// Database connection timeout (in second)
    pub database_idle_timeout: u64,

    /// Redis URL (Ex.: redis://127.0.0.1:6379)
    pub redis_url: String,
    /// Redis keys prefix
    pub redis_prefix: String,
    /// Redis connection_tiemout (in second)
    pub redis_connection_timeout: u64,

    /// SMTP host
    pub smtp_host: String,
    /// SMTP port
    pub smtp_port: u16,
    /// SMTP timeout (in second)
    pub smtp_timeout: u64,
    /// SMTP username
    pub smtp_username: String,
    /// SMTP password
    pub smtp_password: String,

    /// Rate limiter enabled
    pub limiter_enabled: bool,
    /// Rate limiter number of requets per second (-1 for no limit)
    pub limiter_requests_by_second: i64,
    /// Rate limiter expiration time (-1 for no limit)
    pub limiter_expire_in_seconds: i64,
    /// Rate limiter white list
    pub limiter_white_list: String,

    /// Forgotten password expiration duration (in hour)
    pub forgotten_password_expiration_duration: i64,
    /// Forgotten password base URL for link (Ex.: http://localhost)
    pub forgotten_password_base_url: String,
    /// Forgotten password email from
    pub forgotten_password_email_from: String,

    /// Prometheus metics enabled
    pub prometheus_metrics_enabled: bool,

    /// Basic Auth username
    pub basic_auth_username: String,
    /// Basic Auth password
    pub basic_auth_password: String,
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
