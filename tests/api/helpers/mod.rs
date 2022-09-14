pub mod user;

use axum::http::StatusCode;
use serde_json::Value;
use std::collections::HashMap;

///
pub struct TestResponse {
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Value,
}
