//! Custom error module

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use color_eyre::eyre::Result as EyreResult;
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Custom Result type for `AppError`
pub type AppResult<T> = EyreResult<T, AppError>;

/// Represents the custom error message
#[derive(Deserialize, Serialize)]
pub struct AppErrorMessage {
    pub code: u16,
    pub message: String,
}

#[derive(Debug)]
pub enum AppErrorCode {
    InternalError,
    BadRequest,
    NotFound,
    UnprocessableEntity,
    Timeout,
    Unauthorized,
    TooManyRequests,
    MethodNotAllowed,
}

/// Defines available errors
#[derive(Display, Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[display(fmt = "{}", message)]
    InternalError { message: String },

    #[display(fmt = "{}", message)]
    BadRequest { message: String },

    #[display(fmt = "{}", message)]
    NotFound { message: String },

    #[display(fmt = "{}", message)]
    UnprocessableEntity { message: String },

    #[display(fmt = "Request Timeout")]
    Timeout,

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "Too Many Requests")]
    TooManyRequests,

    #[display(fmt = "Method Not Allowed")]
    MethodNotAllowed,
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
            AppError::MethodNotAllowed { .. } => StatusCode::METHOD_NOT_ALLOWED,
            AppError::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
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
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
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

/// Create an [`AppError`] and generate a log if HTTP Code is 500.
///
/// ```rust
/// use axum_boilerplate::utils::errors::{AppError, AppErrorCode, AppResult};
/// use axum_boilerplate::app_error;
///
/// #[macro_use]
/// extern crate tracing;
///
/// fn main() -> AppResult<()> {
///     assert_eq!(AppError::Timeout, app_error!(AppErrorCode::Timeout));
///     assert_eq!(
///         AppError::InternalError{ message: "Internal Server Error".to_owned()},
///         app_error!(AppErrorCode::InternalError)
///     );
///
///     assert_eq!(AppError::Timeout, app_error!(AppErrorCode::Timeout, "Timeout"));
///     assert_eq!(
///         AppError::InternalError{ message: "My error".to_owned()},
///         app_error!(AppErrorCode::InternalError, "My error")
///     );
///
///     assert_eq!(
///         AppError::InternalError{ message: "My error".to_owned()},
///         app_error!(AppErrorCode::InternalError, "My error", "Details of my error")
///     );
///     
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! app_error {
    ( $error:expr ) => {
        match $error {
            AppErrorCode::Timeout => AppError::Timeout,
            AppErrorCode::Unauthorized => AppError::Unauthorized,
            AppErrorCode::TooManyRequests => AppError::TooManyRequests,
            AppErrorCode::MethodNotAllowed => AppError::MethodNotAllowed,
            AppErrorCode::InternalError => AppError::InternalError {
                message: String::from("Internal Server Error"),
            },
            AppErrorCode::BadRequest => AppError::BadRequest {
                message: String::from("Bad Request"),
            },
            AppErrorCode::NotFound => AppError::NotFound {
                message: String::from("Not Found"),
            },
            AppErrorCode::UnprocessableEntity => AppError::UnprocessableEntity {
                message: String::from("Unprocessable Entity"),
            },
        }
    };

    ( $error:expr, $message:expr ) => {
        match $error {
            AppErrorCode::Timeout => AppError::Timeout,
            AppErrorCode::Unauthorized => AppError::Unauthorized,
            AppErrorCode::TooManyRequests => AppError::TooManyRequests,
            AppErrorCode::MethodNotAllowed => AppError::MethodNotAllowed,
            AppErrorCode::InternalError => {
                error!("{}", $message);
                AppError::InternalError {
                    message: $message.to_owned(),
                }
            }
            AppErrorCode::BadRequest => AppError::BadRequest {
                message: $message.to_owned(),
            },
            AppErrorCode::NotFound => AppError::NotFound {
                message: $message.to_owned(),
            },
            AppErrorCode::UnprocessableEntity => AppError::UnprocessableEntity {
                message: $message.to_owned(),
            },
        }
    };

    ( $error:expr, $message:expr, $details:expr ) => {
        match $error {
            AppErrorCode::Timeout => AppError::Timeout,
            AppErrorCode::Unauthorized => AppError::Unauthorized,
            AppErrorCode::TooManyRequests => AppError::TooManyRequests,
            AppErrorCode::MethodNotAllowed => AppError::MethodNotAllowed,
            AppErrorCode::InternalError => {
                error!("{}", $details);
                AppError::InternalError {
                    message: $message.to_owned(),
                }
            }
            AppErrorCode::BadRequest => AppError::BadRequest {
                message: $message.to_owned(),
            },
            AppErrorCode::NotFound => AppError::NotFound {
                message: $message.to_owned(),
            },
            AppErrorCode::UnprocessableEntity => AppError::UnprocessableEntity {
                message: $message.to_owned(),
            },
        }
    };
}
