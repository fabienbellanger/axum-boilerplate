//! JWT layer

use super::body_from_parts;
use crate::{layers, models::auth::Claims};
use axum::{
    body::{boxed, Body, Full},
    http::{Request, StatusCode},
    response::Response,
};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
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

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let is_authorized = match request.extensions().get::<layers::SharedState>() {
            Some(state) => {
                debug!("State");
                let state = state.clone();
                match Claims::extract_from_request(request.headers(), &state.config.jwt_decoding_key.clone()) {
                    Some(claims) => claims.is_ok(),
                    _ => false,
                }
            }
            _ => {
                debug!("No state");
                false
            }
        };

        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response = Response::default();

            response = match is_authorized {
                true => future.await?,
                false => {
                    let (mut parts, _body) = response.into_parts();
                    let msg = body_from_parts(&mut parts, StatusCode::UNAUTHORIZED, "Unauthorized", None);
                    Response::from_parts(parts, boxed(Full::from(msg)))
                }
            };

            Ok(response)
        })
    }
}
