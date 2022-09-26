//! Basic Auth layer

use crate::layers::header_value_to_str;
use axum::{body::Body, http::Request, response::Response};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

// pub struct Credentials {
//     pub username: String,
//     pub paswword: String,
// }

pub struct BadicAuthLayer {
    pub username: String,
    pub password: String,
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
        let _resquest_headers = request.headers();
        dbg!(self.username.clone(), self.password.clone());

        // Use: https://crates.io/crates/http-auth-basic

        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;

            Ok(response)
        })
    }
}
