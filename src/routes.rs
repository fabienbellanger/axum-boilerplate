//! Routes list

use crate::handlers;
use crate::layers;
use axum::routing::{delete, get, post, put};
use axum::Router;

/// Return web routes list
pub fn web() -> Router {
    Router::new()
        .route("/health-check", get(handlers::web::health_check))
        .route("/timeout", get(handlers::web::timeout))
        .route("/spawn", get(handlers::web::spawn))
        .route("/stream", get(handlers::web::stream))
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
    Router::new()
        .route("/", post(handlers::users::create))
        .route("/", get(handlers::users::get_all))
        .route("/:id", get(handlers::users::get_by_id))
        .route("/:id", delete(handlers::users::delete))
        .route("/:id", put(handlers::users::update))
}
