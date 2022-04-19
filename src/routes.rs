//! Routes list

use crate::{handlers, layers};
use axum::{
    routing::{get, post},
    Router,
};

/// Return web routes list
pub fn web() -> Router {
    Router::new().route("/health-check", get(|| async { "OK" }))
}

/// Return API routes list
pub fn api() -> Router {
    Router::new()
        // Public routes
        .route("/login", post(handlers::users::login))
        // Protected routes
        .nest("/", api_protected().layer(layers::jwt::JwtLayer))
}

/// Protected API routes
fn api_protected() -> Router {
    Router::new().nest("/users", api_users())
}

/// Users API routes
fn api_users() -> Router {
    Router::new().route("/", post(handlers::users::register))
}
