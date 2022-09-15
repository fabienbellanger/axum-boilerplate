pub mod user;

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

///
pub struct TestResponse {
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

#[derive(Deserialize)]
pub struct TestUser {
    pub id: String,
    pub lastname: String,
    pub firstname: String,
    pub username: String,
    pub roles: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
