//! Custom error module

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use color_eyre::eyre::Result as EyreResult;
use derive_more::{Display, Error};
use serde::Serialize;
use serde_json::json;

/// Custom Result type for `AppError`
pub type AppResult<T> = EyreResult<T, AppError>;

/// Represents the custom error message
#[derive(Serialize)]
pub struct AppErrorMessage {
    pub code: u16,
    pub message: String,
}

/// Defines available errors
#[derive(Display, Debug, Error)]
pub enum AppError {
    #[display(fmt = "{}", message)]
    InternalError { message: String },

    #[display(fmt = "{}", message)]
    BadRequest { message: String },

    #[display(fmt = "{}", message)]
    NotFound { message: String },

    #[display(fmt = "Request Timeout")]
    Timeout,

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "Too Many Requests")]
    TooManyRequests,
}

impl AppError {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound { message: m } => m.to_owned(),
            Self::BadRequest { message: m } => m.to_owned(),
            Self::InternalError { message: m } => m.to_owned(),
            Self::Unauthorized => "Unauthorized".to_owned(),
            Self::Timeout => "Request Timeout".to_owned(),
            Self::TooManyRequests => "Too Many Requests".to_owned(),
        }
    }
}

// Axum errors
// ------------
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            AppError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            AppError::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
            AppError::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
        };

        let body = Json(json!(AppErrorMessage {
            code: status.as_u16(),
            message: self.to_string(),
        }));

        (status, body).into_response()
    }
}

// SQLx errors
// -----------
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        error!("Database error: {:?}", error);

        Self::InternalError {
            message: "Database Error".to_owned(),
        }
    }
}

// Redis errors
// ------------
impl From<redis::RedisError> for AppError {
    fn from(error: redis::RedisError) -> Self {
        error!("Redis database error: {:?}", error);

        Self::InternalError {
            message: "Redis Database Error".to_owned(),
        }
    }
}

// r2d2 errors
// -----------
impl From<r2d2::Error> for AppError {
    fn from(error: r2d2::Error) -> Self {
        error!("Redis r2d2 pool error: {:?}", error);

        Self::InternalError {
            message: "Redis Database Error".to_owned(),
        }
    }
}

/// Custom Result typefor `CliError`
pub type CliResult<T> = EyreResult<T, CliError>;

/// Custom CLI Error
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CliError {
    #[error("Panic: {0}")]
    Panic(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("CLI error: {0}")]
    Error(String),

    #[error("Server error: {0}")]
    ServerError(String),
}
