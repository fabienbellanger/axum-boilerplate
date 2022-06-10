//! Handlers module

pub mod users;
pub mod web;
#[cfg(feature = "ws")]
pub mod ws;

use crate::errors::AppError;
use axum::{http::StatusCode, response::IntoResponse, BoxError};
use std::io;
use tower::timeout::error::Elapsed;

/// Timeout error
pub async fn timeout_error(err: BoxError) -> AppError {
    if err.is::<Elapsed>() {
        AppError::Timeout {}
    } else {
        AppError::InternalError {
            message: err.to_string(),
        }
    }
}

/// Static file error
pub async fn static_file_error(err: io::Error) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", err),
    )
}
