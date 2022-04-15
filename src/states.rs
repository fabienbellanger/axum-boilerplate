//! Application shared states

use crate::config::Config;
use std::sync::Arc;

/// SharedState
pub type SharedState = Arc<State>;

#[derive(Default, Debug)]
pub struct State {
    pub jwt_secret_key: String,
    pub jwt_lifetime: i64,
}

impl State {
    /// Initialize `State` with configuration data (`.env`)
    pub fn init(config: &Config) -> Self {
        Self {
            jwt_secret_key: config.jwt_secret_key.clone(),
            jwt_lifetime: config.jwt_lifetime,
        }
    }
}
