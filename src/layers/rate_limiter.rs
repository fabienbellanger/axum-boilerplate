//! Rate limiter module

use crate::{
    errors::{AppError, AppResult},
    models::auth::{self, Claims},
};
use axum::{body::Body, extract::ConnectInfo, http::Request, response::Response};
use chrono::{DateTime, Duration, Utc};
use futures::future::BoxFuture;
use r2d2::Pool;
use redis::{Client, Commands};
use std::{
    collections::HashMap,
    net::SocketAddr,
    task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct RateLimiterLayer<'a> {
    pub pool: &'a Pool<Client>,
    pub jwt_secret: String,
    pub enabled: bool,
    pub requests_by_second: i32,
    pub expire_in_seconds: i32,
}

impl<'a> RateLimiterLayer<'a> {
    pub fn new<'b>(
        pool: &'a Pool<Client>,
        jwt_secret: String,
        enabled: bool,
        requests_by_second: i32,
        expire_in_seconds: i32,
    ) -> Self {
        Self {
            pool,
            jwt_secret,
            enabled,
            requests_by_second,
            expire_in_seconds,
        }
    }
}

impl<'a, S> Layer<S> for RateLimiterLayer<'a> {
    type Service = RateLimiterMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimiterMiddleware {
            inner,
            pool: self.pool.clone(),
            jwt_secret: self.jwt_secret.clone(),
            enabled: self.enabled,
            requests_by_second: self.requests_by_second,
            expire_in_seconds: self.expire_in_seconds,
        }
    }
}

#[derive(Clone)]
pub struct RateLimiterMiddleware<S> {
    inner: S,
    pool: Pool<Client>,
    jwt_secret: String,
    enabled: bool,
    requests_by_second: i32,
    expire_in_seconds: i32,
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
        info!("Rate limiter enabled: {}", self.enabled); // TODO: Use it!

        // Redis connection
        let pool = self.pool.clone();
        info!("Redis pool: {:?}", pool);

        // Check JWT claims
        let claims = auth::Claims::extract_from_request(request.headers(), self.jwt_secret.clone());
        let addr = request.extensions().get::<ConnectInfo<SocketAddr>>();

        let result = RateLimiterCheck::init(claims, self.requests_by_second, addr);
        info!("Result={:?}", result);

        result.check_and_update(&pool, self.expire_in_seconds).unwrap(); // TODO: Remove unwrap

        // Headers
        // -------
        // X-Ratelimit-Limit: 50       => limit
        // X-Ratelimit-Remaining: 48   => remaining limit
        // X-Ratelimit-Reset: 17       => remaining seconds

        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;

            Ok(response)
        })
    }
}

#[derive(Debug)]
struct RateLimiterCheck {
    has_error: bool,
    key: Option<String>,
    limit: i32,
}

impl Default for RateLimiterCheck {
    fn default() -> Self {
        Self {
            has_error: false,
            key: None,
            limit: -1,
        }
    }
}

impl RateLimiterCheck {
    fn new(has_error: bool, key: Option<String>, limit: i32) -> Self {
        Self { has_error, key, limit }
    }

    // TODO: Add test
    fn init(claims: Option<Claims>, requests_by_second: i32, addr: Option<&ConnectInfo<SocketAddr>>) -> Self {
        match claims {
            None => {
                let default_limit = requests_by_second;
                if default_limit == -1 {
                    // No limit
                    Self::default()
                } else {
                    // Client Remote IP address
                    match addr {
                        None => Self::new(true, None, 0),
                        Some(remote_address) => {
                            Self::new(false, Some(remote_address.0.ip().to_string()), default_limit)
                        }
                    }
                }
            }
            Some(claims) => Self::new(false, Some(claims.user_id), claims.user_limit),
        }
    }

    /// Checks limit and update Redis
    fn check_and_update(&self, pool: &Pool<Client>, expire_in_seconds: i32) -> AppResult<(i32, i32)> {
        if self.has_error {
            Err(AppError::TooManyRequests)
        } else {
            if self.limit == -1 {
                Ok((0, 0))
            } else {
                let mut conn = pool.get()?;
                let now: DateTime<Utc> = Utc::now();

                // Find if key exists
                let result: HashMap<String, String> = conn.hgetall(&self.key)?;
                dbg!(&result);
                if result.is_empty() {
                    // Not exists
                    conn.hset(&self.key, "remaining", self.limit)?;
                    conn.hset(&self.key, "expiredAt", now.to_rfc3339())?;
                } else {
                    // Exists
                    let _remaining = result.get("remaining").unwrap(); // TODO: Delete unwrap
                    let expired_at = result.get("expiredAt").unwrap(); // TODO: Delete unwrap
                    let expired_at = DateTime::parse_from_rfc3339(expired_at).unwrap();

                    if now >= expired_at {
                        warn!("CACHE EXPIRED: {} / {}", expired_at, now);
                        // Expired
                        let expire_at = now + Duration::seconds(expire_in_seconds as i64);

                        conn.hset(&self.key, "remaining", 50)?;
                        conn.hset(&self.key, "expiredAt", expire_at.to_rfc3339())?;
                    } else {
                        warn!("CACHE VALID: {} / {}", expired_at, now);
                        // Valid
                    }
                }

                let mut current_limit: i32 = 0;
                if current_limit < self.limit {
                    current_limit += 1;
                }

                Ok((current_limit, self.limit))
            }
        }
    }
}
