//! Application layers modules

pub mod basic_auth;
pub mod jwt;
pub mod logger;
pub mod prometheus;
pub mod rate_limiter;

use crate::app_error;
use crate::config::Config;
use crate::utils::errors::{AppError, AppErrorCode, AppErrorMessage};
use axum::body::Body;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
    response::Parts,
    HeaderName, HeaderValue, Method, Request,
};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::StatusCode;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::collections::HashSet;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

/// Construct response body from `Parts`, status code, message and headers
pub fn body_from_parts(
    parts: &mut Parts,
    status_code: StatusCode,
    message: &str,
    headers: Option<Vec<(HeaderName, HeaderValue)>>,
) -> Bytes {
    // Status
    parts.status = status_code;

    // Headers
    parts
        .headers
        .insert(CONTENT_TYPE, HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()));
    if let Some(headers) = headers {
        for header in headers {
            parts.headers.insert(header.0, header.1);
        }
    }

    // Body
    let msg = serde_json::json!(AppErrorMessage {
        code: status_code.as_u16(),
        message: String::from(message),
    });

    Bytes::from(msg.to_string())
}

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

// #[derive(Default, Debug)]
pub struct State {
    pub config: ConfigState,
}

impl State {
    /// Initialize `State` with configuration data (`.env`)
    pub fn init(config: &Config) -> Self {
        info!("Init app state");
        Self {
            config: config.clone().into(),
        }
    }
}

// #[derive(Default, Debug)]
pub struct ConfigState {
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
    pub jwt_lifetime: i64,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_timeout: u64,
    pub forgotten_password_expiration_duration: i64,
    pub forgotten_password_base_url: String,
    pub forgotten_password_email_from: String,
}

impl From<Config> for ConfigState {
    fn from(config: Config) -> Self {
        Self {
            jwt_encoding_key: EncodingKey::from_secret(config.jwt_secret_key.clone().as_bytes()),
            jwt_decoding_key: DecodingKey::from_secret(config.jwt_secret_key.as_bytes()),
            jwt_lifetime: config.jwt_lifetime,
            smtp_host: config.smtp_host.clone(),
            smtp_port: config.smtp_port,
            smtp_timeout: config.smtp_timeout,
            forgotten_password_expiration_duration: config.forgotten_password_expiration_duration,
            forgotten_password_base_url: config.forgotten_password_base_url.clone(),
            forgotten_password_email_from: config.forgotten_password_email_from,
        }
    }
}

pub type SharedChatState = Arc<ChatState>;

/// State for WebSocket chat example
pub struct ChatState {
    pub user_set: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<String>,
}

// ================ CORS ================

/// CORS layer
pub fn cors(config: &Config) -> CorsLayer {
    let allow_origin = config.cors_allow_origin.clone();

    let layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, ORIGIN, CONTENT_TYPE]);

    if allow_origin == "*" {
        layer.allow_origin(Any)
    } else {
        let origins = allow_origin
            .split(',')
            .filter(|url| *url != "*" && !url.is_empty())
            .filter_map(|url| url.parse().ok())
            .collect::<Vec<HeaderValue>>();

        if origins.is_empty() {
            layer.allow_origin(Any)
        } else {
            layer
                .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _| {
                    origins.contains(origin)
                }))
                .allow_credentials(true)
        }
    }
}

// =============== Override some HTTP errors ================

/// Layer which override some HTTP errors by using `AppError`
pub async fn override_http_errors(req: Request<Body>, next: Next) -> impl IntoResponse {
    let response = next.run(req).await;

    // If it is an image, audio or video, we return response
    let headers = response.headers();
    if let Some(content_type) = headers.get("content-type") {
        let content_type = content_type.to_str().unwrap_or_default();
        if content_type.starts_with("image/")
            || content_type.starts_with("audio/")
            || content_type.starts_with("video/")
        {
            return response;
        }
    }

    let (parts, body) = response.into_parts();
    match body.collect().await {
        Ok(body_bytes) => match String::from_utf8(body_bytes.to_bytes().to_vec()) {
            Ok(body) => match parts.status {
                StatusCode::METHOD_NOT_ALLOWED => app_error!(AppErrorCode::MethodNotAllowed).into_response(),
                StatusCode::UNPROCESSABLE_ENTITY => app_error!(AppErrorCode::UnprocessableEntity, body).into_response(),
                _ => Response::from_parts(parts, Body::from(body)),
            },
            Err(err) => app_error!(AppErrorCode::InternalError, err.to_string()).into_response(),
        },
        Err(err) => {
            return app_error!(AppErrorCode::InternalError, err.to_string()).into_response();
        }
    }
}

// =============== Utils ================

/// Convert `HeaderValue` to `&str`
pub fn header_value_to_str(value: Option<&HeaderValue>) -> &str {
    match value {
        Some(value) => from_utf8(value.as_bytes()).unwrap_or_default(),
        None => "",
    }
}
