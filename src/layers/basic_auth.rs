//! Basic Auth layer

use super::body_from_parts;
use axum::{
    body::{boxed, Body, Full},
    http::{header, HeaderValue, Request},
    response::Response,
};
use futures::future::BoxFuture;
use http_auth_basic::Credentials;
use hyper::StatusCode;
use std::task::{Context, Poll};
use tower::{Layer, Service};

pub struct BasicAuthLayer {
    pub username: String,
    pub password: String,
}

impl BasicAuthLayer {
    /// Create a new `BasicAuthLayer`
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

impl<S> Layer<S> for BasicAuthLayer {
    type Service = BasicAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BasicAuthMiddleware {
            inner,
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }
}

#[derive(Clone)]
pub struct BasicAuthMiddleware<S> {
    inner: S,
    username: String,
    password: String,
}

impl<S> Service<Request<Body>> for BasicAuthMiddleware<S>
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
        let auth = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(str::to_string);
        let username = self.username.clone();
        let password = self.password.clone();

        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response = Response::default();

            let ok = match auth {
                None => false,
                Some(auth) => match Credentials::from_header(auth) {
                    Err(_) => false,
                    Ok(cred) => cred.user_id == username && cred.password == password,
                },
            };
            response = match ok {
                true => future.await?,
                false => {
                    let (mut parts, _body) = response.into_parts();
                    let msg = body_from_parts(
                        &mut parts,
                        StatusCode::UNAUTHORIZED,
                        "Unauthorized",
                        Some(vec![(
                            header::WWW_AUTHENTICATE,
                            HeaderValue::from_static("basic realm=RESTRICTED"),
                        )]),
                    );
                    Response::from_parts(parts, boxed(Full::from(msg)))
                }
            };

            Ok(response)
        })
    }
}
