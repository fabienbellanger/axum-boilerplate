//! Application layers

use crate::config::Config;
use axum::http::{HeaderValue, Method, Request};
use std::str::from_utf8;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

// ================ Request ID ================

// Request ID middleware
// TODO: Put in middlewares module
#[derive(Clone, Copy)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string().parse().unwrap();
        Some(RequestId::new(request_id))
    }
}

/// Convert `HeaderValue` to `&str`
pub fn header_value_to_str(value: Option<&HeaderValue>) -> &str {
    match value {
        Some(value) => from_utf8(value.as_bytes()).unwrap_or(""),
        None => "",
    }
}

// ================ States ================

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

// ================ CORS ================

/// CORS layer
pub fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_origin(Any)
}
