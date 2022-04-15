//! Routes list

use crate::handlers;
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
        .nest(
            "/",
            Router::new()
                .route("/register", post(handlers::users::register))
                .layer(crate::middlewares::jwt::JwtLayer),
        )
}
