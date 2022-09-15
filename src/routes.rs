//! Routes list

use crate::handlers;
use crate::layers;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

/// Return web routes list
pub fn web() -> Router {
    Router::new()
        .route("/health-check", get(handlers::web::health_check))
        .route("/hello", get(handlers::web::hello))
        .route("/timeout", get(handlers::web::timeout))
        .route("/spawn", get(handlers::web::spawn))
        // Test of streams and large data
        .route("/big-json", get(handlers::web::big_json))
        .route("/stream", get(handlers::web::stream))
}

/// Return WebSocket routes list
pub fn ws() -> Router {
    Router::new()
        .route("/", get(handlers::ws::simple_ws_handler))
        .route("/chat", get(handlers::ws::chat_ws_handler))
}

/// Return API routes list
pub fn api() -> Router {
    Router::new()
        // Public routes
        .route("/login", post(handlers::users::login))
        .route("/forgotten-password/:email", post(handlers::users::forgotten_password))
        .route("/update-password/:token", patch(handlers::users::update_password))
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
