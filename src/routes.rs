//! Routes list

use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};

/// Return web routes list
pub fn list() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health-check", get(|| async { "OK" }))
}

/// Return API routes list
pub fn api() -> Router {
    Router::new()
        .route("/login", post(handlers::users::login))
        .route("/register", post(handlers::users::register))
}
