//! Application layers modules

pub mod jwt;
pub mod logger;

use crate::config::Config;
use axum::http::{HeaderValue, Method, Request, Response};
use std::str::from_utf8;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

// ================ Request ID ================

/// Request ID middleware
#[derive(Clone, Copy)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string().parse();
        match id {
            Ok(id) => Some(RequestId::new(id)),
            _ => None,
        }
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

// =============== Utils ================

/// Convert `HeaderValue` to `&str`
// TODO: Create a module utils
pub fn header_value_to_str(value: Option<&HeaderValue>) -> &str {
    match value {
        Some(value) => from_utf8(value.as_bytes()).unwrap_or(""),
        None => "",
    }
}

// TODO
pub fn layer_response<T>(_response: &mut Response<T>) {
    /*
        let (mut parts, _body) = response.into_parts();

        // Status code
        parts.status = StatusCode::UNAUTHORIZED;

        // Content Type
        parts.headers.insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        // Body
        let msg = serde_json::json!(AppErrorMessage {
            code: StatusCode::UNAUTHORIZED.as_u16(),
            message: String::from("Unauthorized"),
        });
        let msg = Bytes::from(msg.to_string());

        Response::from_parts(parts, axum::body::boxed(Full::from(msg)))
    */
}
