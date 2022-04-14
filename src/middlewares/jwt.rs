//! JWT middleware

use crate::{errors::AppError, models::auth};
use axum::{body::Body, http::Request, response::Response};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

pub struct JwtLayer;

impl<S> Layer<S> for JwtLayer {
    type Service = JwtMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        JwtMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct JwtMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for JwtMiddleware<S>
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

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        // let mut is_authorized = false;

        if let Some(claims) = auth::Claims::extract_from_request(request.headers(), "mySecretKey".to_owned()) {
            warn!("{:?}", claims);
            // is_authorized = true;
        }

        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
