//! Handlers module

pub mod users;

use crate::errors::AppError;
use axum::BoxError;
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
