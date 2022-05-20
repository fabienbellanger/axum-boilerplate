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

pub struct RateLimiterLayer<'a> {
    pub pool: &'a Pool<Client>,
    pub jwt_secret: String,
}

impl<'a> RateLimiterLayer<'a> {
    pub fn new<'b>(pool: &'a Pool<Client>, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }
}

impl<'a, S> Layer<S> for RateLimiterLayer<'a> {
    type Service = RateLimiterMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimiterMiddleware {
            inner,
            pool: self.pool.clone(),
            jwt_secret: self.jwt_secret.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimiterMiddleware<S> {
    inner: S,
    pool: Pool<Client>,
    jwt_secret: String,
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
        let pool = self.pool.clone();
        info!("Redis pool: {:?}", pool);

        // JWT
        let _jwt_secret = self.jwt_secret.clone();
        info!("JWT secret: {}", _jwt_secret);
        // <--------

        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;

            Ok(response)
        })
    }
}
