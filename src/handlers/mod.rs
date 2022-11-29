//! Handlers module

pub mod users;
pub mod web;
pub mod ws;

use crate::{
    app_error,
    errors::{AppError, AppErrorCode, AppResult},
};
use axum::http::StatusCode;
use axum::BoxError;
use std::io;
use tower::timeout::error::Elapsed;

/// Timeout error
// pub async fn timeout_error(err: BoxError) -> AppResult<()> {
//     if err.is::<Elapsed>() {
//         Err(app_error!(AppErrorCode::Timeout))
//     } else {
//         Err(app_error!(AppErrorCode::InternalError, err.to_string()))
//     }
// }
pub async fn timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Request took too long".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}

/// Static file error
pub async fn static_file_error(err: io::Error) -> AppResult<()> {
    Err(app_error!(
        AppErrorCode::InternalError,
        "error when serving static file",
        format!("Unhandled internal error: {err}")
    ))
}
