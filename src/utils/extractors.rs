//! Custom Axum extractors

use super::errors::{AppError, AppErrorCode};
use crate::app_error;
use axum::http::{header::HeaderValue, request::Parts};
use axum::{
    async_trait,
    extract::{path::ErrorKind, rejection::PathRejection, FromRequestParts},
};
use hyper::StatusCode;
use serde::de::DeserializeOwned;

/// Request ID extractor from HTTP headers
pub struct ExtractRequestId(pub HeaderValue);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractRequestId
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.headers.get("x-request-id") {
            Some(id) => Ok(ExtractRequestId(id.clone())),
            _ => Ok(ExtractRequestId(HeaderValue::from_static(""))),
        }
    }
}

// We define our own `Path` extractor that customizes the error from `axum::extract::Path`
pub struct Path<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Path<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = (StatusCode, AppError);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, body) = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        let mut status = StatusCode::BAD_REQUEST;

                        let kind = inner.into_kind();
                        let body = match &kind {
                            ErrorKind::WrongNumberOfParameters { .. } => {
                                app_error!(AppErrorCode::BadRequest, kind.to_string())
                            }

                            ErrorKind::ParseErrorAtKey { .. } => app_error!(AppErrorCode::BadRequest, kind.to_string()),

                            ErrorKind::ParseErrorAtIndex { .. } => {
                                app_error!(AppErrorCode::BadRequest, kind.to_string())
                            }

                            ErrorKind::ParseError { .. } => app_error!(AppErrorCode::BadRequest, kind.to_string()),

                            ErrorKind::InvalidUtf8InPathParam { .. } => {
                                app_error!(AppErrorCode::BadRequest, kind.to_string())
                            }

                            ErrorKind::UnsupportedType { .. } => {
                                // this error is caused by the programmer using an unsupported type
                                // (such as nested maps) so respond with `500` instead
                                status = StatusCode::INTERNAL_SERVER_ERROR;
                                app_error!(AppErrorCode::InternalError, kind.to_string())
                            }

                            ErrorKind::Message(msg) => app_error!(AppErrorCode::BadRequest, msg.clone()),

                            _ => app_error!(
                                AppErrorCode::BadRequest,
                                format!("Unhandled deserialization error: {}", kind)
                            ),
                        };

                        (status, body)
                    }
                    PathRejection::MissingPathParams(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        app_error!(AppErrorCode::InternalError, error.to_string()),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        app_error!(
                            AppErrorCode::InternalError,
                            format!("Unhandled path rejection: {}", rejection)
                        ),
                    ),
                };

                Err((status, body))
            }
        }
    }
}
