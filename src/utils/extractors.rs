//! Custom Axum extractors

use crate::errors::AppError;
use axum::http::header::HeaderValue;
use axum::{
    async_trait,
    extract::{path::ErrorKind, rejection::PathRejection, FromRequest, RequestParts},
};
use hyper::StatusCode;
use serde::de::DeserializeOwned;

/// Request ID extractor from HTTP headers
pub struct ExtractRequestId(pub HeaderValue);

#[async_trait]
impl<B> FromRequest<B> for ExtractRequestId
where
    B: Send,
{
    type Rejection = ();

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match req.headers().get("x-request-id") {
            Some(id) => Ok(ExtractRequestId(id.clone())),
            _ => Ok(ExtractRequestId(HeaderValue::from_static(""))),
        }
    }
}

// We define our own `Path` extractor that customizes the error from `axum::extract::Path`
pub struct Path<T>(pub T);

// TODO: Change when axum 0.6 will release
// https://github.com/tokio-rs/axum/blob/main/examples/customize-path-rejection/src/main.rs
#[async_trait]
impl<B, T> FromRequest<B> for Path<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
    T: DeserializeOwned + Send,
    B: Send,
{
    type Rejection = (StatusCode, AppError);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, body) = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        let mut status = StatusCode::BAD_REQUEST;

                        let kind = inner.into_kind();
                        let body = match &kind {
                            ErrorKind::WrongNumberOfParameters { .. } => AppError::BadRequest {
                                message: kind.to_string(),
                            },

                            ErrorKind::ParseErrorAtKey { .. } => AppError::BadRequest {
                                message: kind.to_string(),
                            },

                            ErrorKind::ParseErrorAtIndex { .. } => AppError::BadRequest {
                                message: kind.to_string(),
                            },

                            ErrorKind::ParseError { .. } => AppError::BadRequest {
                                message: kind.to_string(),
                            },

                            ErrorKind::InvalidUtf8InPathParam { .. } => AppError::BadRequest {
                                message: kind.to_string(),
                            },

                            ErrorKind::UnsupportedType { .. } => {
                                // this error is caused by the programmer using an unsupported type
                                // (such as nested maps) so respond with `500` instead
                                status = StatusCode::INTERNAL_SERVER_ERROR;
                                AppError::InternalError {
                                    message: kind.to_string(),
                                }
                            }

                            ErrorKind::Message(msg) => AppError::BadRequest { message: msg.clone() },

                            _ => AppError::BadRequest {
                                message: format!("Unhandled deserialization error: {}", kind),
                            },
                        };

                        (status, body)
                    }
                    PathRejection::MissingPathParams(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AppError::InternalError {
                            message: error.to_string(),
                        },
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AppError::InternalError {
                            message: format!("Unhandled path rejection: {}", rejection),
                        },
                    ),
                };

                Err((status, body))
            }
        }
    }
}
