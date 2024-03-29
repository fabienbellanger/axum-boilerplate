//! Handlers module

pub mod users;
pub mod web;
pub mod ws;

use crate::{
    app_error,
    utils::errors::{AppError, AppErrorCode, AppResult},
};
use axum::BoxError;
use std::io;
use tower::timeout::error::Elapsed;

/// Timeout error
pub async fn timeout_error(err: BoxError) -> AppResult<()> {
    if err.is::<Elapsed>() {
        Err(app_error!(AppErrorCode::Timeout))
    } else {
        Err(app_error!(AppErrorCode::InternalError, err.to_string()))
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
