//! JWT layer

use crate::{errors::AppErrorMessage, layers, models::auth};
use axum::{
    body::{Body, Full},
    http::{HeaderValue, Request, StatusCode},
    response::Response,
};
use bytes::Bytes;
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

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let is_authorized = match request.extensions().get::<layers::SharedState>() {
            Some(state) => {
                let state = state.clone();
                auth::Claims::extract_from_request(request.headers(), state.jwt_secret_key.clone()).is_some()
            }
            _ => false,
        };

        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response = Response::default();

            response = match is_authorized {
                true => future.await?,
                false => {
                    let (mut parts, _body) = response.into_parts();

                    // Status
                    parts.status = StatusCode::UNAUTHORIZED;

                    // Content Type
                    parts.headers.insert(
                        axum::http::header::CONTENT_TYPE,
                        HeaderValue::from_static("application/json"),
                    );

                    // Body
                    let msg = serde_json::json!(AppErrorMessage {
                        code: StatusCode::UNAUTHORIZED.as_u16(),
                        message: String::from("Unauthorized"),
                    });
                    let msg = Bytes::from(msg.to_string());

                    Response::from_parts(parts, axum::body::boxed(Full::from(msg)))
                }
            };

            Ok(response)
        })
    }
}
