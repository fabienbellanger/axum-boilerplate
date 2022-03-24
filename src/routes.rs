//! Routes list

use axum::{routing::get, Router};

/// Return routes list
pub fn list() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/hi", get(|| async { "Hi!" }))
}
