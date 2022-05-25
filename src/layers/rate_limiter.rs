//! Rate limiter module

use crate::models::auth::{self, Claims};
use axum::{body::Body, extract::ConnectInfo, http::Request, response::Response};
use chrono::{DateTime, Duration, Utc};
use derive_more::{Display, Error};
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
    pub fn new(
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

        let t = result.check_and_update(&pool, self.expire_in_seconds);
        warn!("t={:?}", t);

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

#[derive(Display, Debug, Error, Copy, Clone)]
enum RateLimiterError {
    Ip,
    Redis,
    DateTime,
    Parse,
}

impl From<redis::RedisError> for RateLimiterError {
    fn from(_error: redis::RedisError) -> Self {
        Self::Redis {}
    }
}

impl From<r2d2::Error> for RateLimiterError {
    fn from(_error: r2d2::Error) -> Self {
        Self::Redis {}
    }
}

#[derive(Debug)]
struct RateLimiterCheck {
    error: Option<RateLimiterError>,
    key: Option<String>,
    limit: i32,
}

impl Default for RateLimiterCheck {
    fn default() -> Self {
        Self {
            error: None,
            key: None,
            limit: -1,
        }
    }
}

impl RateLimiterCheck {
    fn new(error: Option<RateLimiterError>, key: Option<String>, limit: i32) -> Self {
        Self { error, key, limit }
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
                        None => Self::new(Some(RateLimiterError::Ip), None, 0),
                        Some(remote_address) => Self::new(None, Some(remote_address.0.ip().to_string()), default_limit),
                    }
                }
            }
            Some(claims) => Self::new(None, Some(claims.user_id), claims.user_limit),
        }
    }

    /// Checks limit, update Redis and reurns information for headers
    fn check_and_update(&self, pool: &Pool<Client>, expire_in_seconds: i32) -> Result<(i32, i64), RateLimiterError> {
        if let Some(err) = self.error {
            return Err(err);
        } else if self.limit == -1 {
            Ok((0, 0))
        } else {
            let mut conn = pool.get()?;
            let now: DateTime<Utc> = Utc::now();
            let mut remaining = self.limit - 1;
            let mut reset = expire_in_seconds as i64;
            let mut expired_at = now + Duration::seconds(expire_in_seconds as i64);

            let result: HashMap<String, String> = conn.hgetall(&self.key)?;

            if !result.is_empty() {
                let expired_at_str = result.get("expiredAt").ok_or(RateLimiterError::Redis)?;
                expired_at = DateTime::parse_from_rfc3339(expired_at_str)
                    .map_err(|_err| RateLimiterError::DateTime)?
                    .with_timezone(&Utc);

                reset = (expired_at - now).num_seconds();

                if reset <= 0 {
                    // Expired cache
                    // -------------
                    conn.del(&self.key)?; // TODO: Necesary?

                    expired_at = now + Duration::seconds(expire_in_seconds as i64);
                    reset = expire_in_seconds as i64;
                } else {
                    // Valid cache
                    // -----------
                    let remaining_str = result.get("remaining").ok_or(RateLimiterError::Redis)?;
                    remaining = remaining_str.parse::<i32>().map_err(|_err| RateLimiterError::Parse)?;

                    if remaining > 0 {
                        remaining -= 1;
                    }
                }
            }

            conn.hset(&self.key, "remaining", remaining)?;
            conn.hset(&self.key, "expiredAt", expired_at.to_rfc3339())?;

            Ok((remaining, reset))
        }
    }
}
