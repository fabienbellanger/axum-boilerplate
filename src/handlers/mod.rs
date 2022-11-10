//! Handlers module

pub mod users;
pub mod web;
pub mod ws;

use crate::{
    app_error,
    errors::{AppError, AppErrorCode},
};
use axum::{http::StatusCode, response::IntoResponse, BoxError};
use std::io;
use tower::timeout::error::Elapsed;

/// Timeout error
pub async fn timeout_error(err: BoxError) -> AppError {
    if err.is::<Elapsed>() {
        app_error!(AppErrorCode::Timeout)
    } else {
        app_error!(AppErrorCode::InternalError, err.to_string())
    }
}

/// Static file error
pub async fn static_file_error(err: io::Error) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled internal error: {}", err),
    )
}
