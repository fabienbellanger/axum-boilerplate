//! Rate limiter module

use axum::{body::Body, extract::ConnectInfo, http::Request, response::Response};
use futures::future::BoxFuture;
use r2d2::Pool;
use redis::Client;
use std::{
    net::SocketAddr,
    task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct RateLimiterLayer;

impl<S> Layer<S> for RateLimiterLayer {
    type Service = RateLimiterMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimiterMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct RateLimiterMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for RateLimiterMiddleware<S>
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
        // --------> TODO: Only for test
        // Client Remote IP address
        let remote_address = request.extensions().get::<ConnectInfo<SocketAddr>>().unwrap();
        let remote_address = remote_address.0.ip().to_string();
        info!("Client address: {:?}", remote_address);

        // Redis connection
        let _pool = request.extensions().get::<Pool<Client>>().unwrap();
        // <--------

        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;

            Ok(response)
        })
    }
}
