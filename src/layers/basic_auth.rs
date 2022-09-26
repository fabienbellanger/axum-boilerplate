//! Basic Auth layer

use axum::{
    body::{Body, Full},
    http::{header, response::Parts, HeaderValue, Request},
    response::Response,
};
use bytes::Bytes;
use futures::future::BoxFuture;
use http_auth_basic::Credentials;
use hyper::StatusCode;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::errors::AppErrorMessage;

pub struct BadicAuthLayer {
    pub username: String,
    pub password: String, // Option<String> ?
}

impl<S> Layer<S> for BadicAuthLayer {
    type Service = BadicAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BadicAuthMiddleware {
            inner,
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }
}

#[derive(Clone)]
pub struct BadicAuthMiddleware<S> {
    inner: S,
    username: String,
    password: String,
}

impl<S> Service<Request<Body>> for BadicAuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,

    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        // let auth_header_value = String::from("Basic dXNlcm5hbWU6cGFzc3dvcmQ=");
        // let credentials = Credentials::from_header(auth_header_value).unwrap();
        // dbg!(credentials);

        // Use: https://crates.io/crates/http-auth-basic

        let auth = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let username = self.username.clone();
        let password = self.password.clone();

        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response = Response::default();

            // TODO: Improve code!
            response = match auth.is_empty() {
                true => {
                    let (mut parts, _body) = response.into_parts();
                    let message = from_parts(&mut parts, StatusCode::UNAUTHORIZED, "Unauthorized");
                    Response::from_parts(parts, axum::body::boxed(Full::from(message)))
                }
                false => match Credentials::from_header(auth.to_string()) {
                    Err(_) => {
                        let (mut parts, _body) = response.into_parts();
                        let message = from_parts(&mut parts, StatusCode::UNAUTHORIZED, "Unauthorized");
                        Response::from_parts(parts, axum::body::boxed(Full::from(message)))
                    }
                    Ok(credentials) => {
                        if credentials.user_id == username && credentials.password == password {
                            future.await?
                        } else {
                            let (mut parts, _body) = response.into_parts();
                            let message = from_parts(&mut parts, StatusCode::UNAUTHORIZED, "Unauthorized");
                            Response::from_parts(parts, axum::body::boxed(Full::from(message)))
                        }
                    }
                },
            };

            Ok(response)
        })
    }
}

// TODO: Generic?
fn from_parts(parts: &mut Parts, status_code: StatusCode, message: &str) -> Bytes {
    // Status
    parts.status = status_code;

    // Content Type
    parts.headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
    );
    if status_code == StatusCode::UNAUTHORIZED {
        parts.headers.insert(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("basic realm=RESTRICTED"),
        );
    }

    // Body
    let msg = serde_json::json!(AppErrorMessage {
        code: status_code.as_u16(),
        message: String::from(message),
    });
    Bytes::from(msg.to_string())
}
