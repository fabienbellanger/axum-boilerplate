//! Rate limiter middleware

use super::{body_from_parts, SharedState};
use crate::{
    errors::AppError,
    models::auth::{self, Claims},
};
use axum::{
    body::{Body, Full},
    extract::ConnectInfo,
    http::{response::Parts, HeaderValue, Request, StatusCode},
    response::Response,
};
use chrono::Utc;
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

const RATE_LIMITER_PREFIX: &str = "rl_";
const LIMIT_HEADER: &str = "x-ratelimit-limit";
const REMAINING_HEADER: &str = "x-ratelimit-remaining";
const RESET_HEADER: &str = "x-ratelimit-reset";
const RETRY_AFTER_HEADER: &str = "retry-after";

#[derive(Clone)]
pub struct RateLimiterLayer {
    pub pool: Pool<Client>,
    pub redis_prefix: String,
    pub requests_by_second: i32,
    pub expire_in_seconds: i64,
    pub white_list: String,
}

impl RateLimiterLayer {
    pub fn new(
        pool: Pool<Client>,
        redis_prefix: String,
        requests_by_second: i32,
        expire_in_seconds: i64,
        white_list: String,
    ) -> Self {
        let mut redis_prefix = redis_prefix;
        redis_prefix.push_str(RATE_LIMITER_PREFIX);

        Self {
            pool,
            redis_prefix,
            requests_by_second,
            expire_in_seconds,
            white_list,
        }
    }
}

impl<S> Layer<S> for RateLimiterLayer {
    type Service = RateLimiterMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let white_list = self.white_list.split(',').map(|s| s.to_string()).collect();

        RateLimiterMiddleware {
            inner,
            pool: self.pool.clone(),
            redis_prefix: self.redis_prefix.clone(),
            requests_by_second: self.requests_by_second,
            expire_in_seconds: self.expire_in_seconds,
            white_list,
        }
    }
}

#[derive(Clone)]
pub struct RateLimiterMiddleware<S> {
    inner: S,
    pool: Pool<Client>,
    redis_prefix: String,
    requests_by_second: i32,
    expire_in_seconds: i64,
    white_list: Vec<String>,
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
        // Redis connection
        let pool = self.pool.clone();

        // Check JWT claims
        let state = request.extensions().get::<SharedState>();
        let claims = match state {
            Some(state) => {
                auth::Claims::extract_from_request(request.headers(), &state.config.jwt_decoding_key.clone())
            }
            None => None,
        };

        // Get socket address
        let addr = request.extensions().get::<ConnectInfo<SocketAddr>>();

        // Initialize RateLimiterCheck
        let check = RateLimiterCheck::init(
            claims,
            addr,
            &self.white_list,
            &self.redis_prefix,
            self.requests_by_second,
        );
        let check_result = check.process(&pool, self.expire_in_seconds);

        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response = Response::default();

            response = match check_result {
                Ok((limit, _remaining, _reset)) if limit <= -1 => future.await?,
                Ok((limit, remaining, reset)) if remaining >= 0 => {
                    // Limit OK
                    // --------
                    let (mut parts, body) = future.await?.into_parts();

                    set_headers(&mut parts, limit, remaining, reset);

                    Response::from_parts(parts, body)
                }
                Ok((limit, remaining, reset)) => {
                    // Limit KO
                    // --------
                    let (mut parts, _body) = response.into_parts();

                    // Headers
                    set_headers(&mut parts, limit, remaining, reset);

                    let msg = body_from_parts(&mut parts, StatusCode::TOO_MANY_REQUESTS, "Too Many Requests", None);
                    Response::from_parts(parts, axum::body::boxed(Full::from(msg)))
                }
                Err(err) => match err {
                    RateLimiterError::JwtDecoding => {
                        let (mut parts, _body) = response.into_parts();
                        let msg = body_from_parts(&mut parts, StatusCode::UNAUTHORIZED, "Unauthorized", None);
                        Response::from_parts(parts, axum::body::boxed(Full::from(msg)))
                    }
                    _ => {
                        let (mut parts, _body) = response.into_parts();
                        let msg =
                            body_from_parts(&mut parts, StatusCode::INTERNAL_SERVER_ERROR, &err.to_string(), None);
                        Response::from_parts(parts, axum::body::boxed(Full::from(msg)))
                    }
                },
            };

            Ok(response)
        })
    }
}

/// Set middleware specific headers
fn set_headers(parts: &mut Parts, limit: i32, remaining: i64, reset: i64) {
    if remaining >= 0 {
        // Limit OK
        if let Ok(limit) = HeaderValue::from_str(limit.to_string().as_str()) {
            parts.headers.insert(LIMIT_HEADER, limit);
        }

        if let Ok(remaining) = HeaderValue::from_str(remaining.to_string().as_str()) {
            parts.headers.insert(REMAINING_HEADER, remaining);
        }

        if let Ok(reset) = HeaderValue::from_str(reset.to_string().as_str()) {
            parts.headers.insert(RESET_HEADER, reset);
        }
    } else {
        // Limit reached
        if let Ok(reset) = HeaderValue::from_str(reset.to_string().as_str()) {
            parts.headers.insert(RETRY_AFTER_HEADER, reset);
        }
    }
}

#[derive(Display, Debug, Error, Clone, PartialEq)]
enum RateLimiterError {
    Ip,
    JwtDecoding,

    #[display(fmt = "{}", message)]
    Redis {
        message: String,
    },
}

impl From<redis::RedisError> for RateLimiterError {
    fn from(error: redis::RedisError) -> Self {
        error!("Redis database error from Rate Limiter middleware: {:?}", error);

        Self::Redis {
            message: error.to_string(),
        }
    }
}

impl From<r2d2::Error> for RateLimiterError {
    fn from(error: r2d2::Error) -> Self {
        error!("Redis r2d2 pool error from Rate Limiter middleware: {:?}", error);

        Self::Redis {
            message: error.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct RateLimiterCheck {
    /// Potential error during the check
    error: Option<RateLimiterError>,

    /// Key used by the uniqueness of consumer
    key: Option<String>,

    /// Request limit (-1: unlimited, 0: when error, >=1: request limit)
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
    /// Create a new instance of `RateLimiterCheck`
    fn new(error: Option<RateLimiterError>, key: Option<String>, limit: i32) -> Self {
        Self { error, key, limit }
    }

    /// Initialize `RateLimiterCheck`
    fn init(
        claims: Option<Result<Claims, AppError>>,
        addr: Option<&ConnectInfo<SocketAddr>>,
        white_list: &[String],
        redis_prefix: &str,
        requests_by_second: i32,
    ) -> Self {
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
                        Some(remote_address) => {
                            let mut key = remote_address.0.ip().to_string();

                            // Check if IP address is in white list
                            if white_list.contains(&key) {
                                // No limit
                                Self::default()
                            } else {
                                key.insert_str(0, redis_prefix);

                                Self::new(None, Some(key), default_limit)
                            }
                        }
                    }
                }
            }
            Some(claims) => match claims {
                Ok(claims) => {
                    if claims.user_rate_limit == -1 {
                        // No limit
                        Self::default()
                    } else {
                        let mut key = claims.user_id;
                        key.insert_str(0, redis_prefix);

                        Self::new(None, Some(key), claims.user_rate_limit)
                    }
                }
                _ => Self::new(Some(RateLimiterError::JwtDecoding), None, 0),
            },
        }
    }

    /// Check limit, update Redis and returns information for headers
    fn process(&self, pool: &Pool<Client>, expire_in_seconds: i64) -> Result<(i32, i64, i64), RateLimiterError> {
        if let Some(err) = &self.error {
            Err(err.clone())
        } else if self.limit == -1 {
            Ok((self.limit, 0, 0))
        } else {
            let mut conn = pool.get()?;
            let now = Utc::now().timestamp();
            let mut remaining = self.limit as i64 - 1;
            let mut reset = expire_in_seconds;
            let mut expired_at = now + expire_in_seconds;

            let result: HashMap<String, i64> = conn.hgetall(&self.key)?;

            if !result.is_empty() {
                expired_at = *result.get("expiredAt").ok_or(RateLimiterError::Redis {
                    message: "Redis key not found in Rate Limiter middleware".to_owned(),
                })?;
                reset = expired_at - now;

                if reset <= 0 {
                    // Expired cache
                    // -------------
                    expired_at = now + expire_in_seconds;
                    reset = expire_in_seconds;
                } else {
                    // Valid cache
                    // -----------
                    remaining = *result.get("remaining").ok_or(RateLimiterError::Redis {
                        message: "Redis key not found in Rate Limiter middleware".to_owned(),
                    })?;

                    if remaining >= 0 {
                        remaining -= 1;
                    }
                }
            }

            conn.hset(&self.key, "remaining", remaining)?;
            conn.hset(&self.key, "expiredAt", expired_at)?;

            Ok((self.limit, remaining, reset))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{app_error, errors::AppErrorCode};

    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_rate_limiter_check_init() {
        let mut requests_by_second = 30;
        let claims = None;
        let addr = None;
        let redis_prefix = "axum_rl_";

        assert_eq!(
            RateLimiterCheck::init(claims, addr, &vec![], redis_prefix, requests_by_second),
            RateLimiterCheck {
                error: Some(RateLimiterError::Ip),
                key: None,
                limit: 0
            }
        );

        requests_by_second = -1;
        let claims = None;
        let addr = None;
        assert_eq!(
            RateLimiterCheck::init(claims, addr, &vec![], redis_prefix, requests_by_second),
            RateLimiterCheck {
                error: None,
                key: None,
                limit: -1
            }
        );

        requests_by_second = 30;
        let user_id = uuid::Uuid::new_v4().to_string();
        let claims = Some(Ok(Claims {
            sub: String::from("Subject"),
            exp: 123456789,
            iat: 123456789,
            nbf: 123456789,
            user_id: user_id.clone(),
            user_roles: String::from("ADMIN"),
            user_rate_limit: 25,
        }));
        let addr = None;
        let mut key = user_id.clone();
        key.insert_str(0, redis_prefix);
        assert_eq!(
            RateLimiterCheck::init(claims, addr, &vec![], redis_prefix, requests_by_second),
            RateLimiterCheck {
                error: None,
                key: Some(key),
                limit: 25
            }
        );

        let claims = None;
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
        let addr = Some(ConnectInfo(socket));
        let mut key = "127.0.0.1".to_owned();
        key.insert_str(0, redis_prefix);
        assert_eq!(
            RateLimiterCheck::init(claims, addr.as_ref(), &vec![], redis_prefix, requests_by_second),
            RateLimiterCheck {
                error: None,
                key: Some(key),
                limit: 30
            }
        );
    }

    #[test]
    fn test_rate_limiter_check_init_with_white_list() {
        let requests_by_second = 30;
        let redis_prefix = "axum_rl_";
        let claims = None;
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
        let addr = Some(ConnectInfo(socket));
        let mut key = "127.0.0.1".to_owned();
        key.insert_str(0, redis_prefix);

        assert_eq!(
            RateLimiterCheck::init(
                claims,
                addr.as_ref(),
                &vec!["127.0.0.1".to_owned()],
                redis_prefix,
                requests_by_second
            ),
            RateLimiterCheck {
                error: None,
                key: None,
                limit: -1
            }
        );
    }

    #[test]
    fn test_rate_limiter_check_init_with_jwt_decoding_error() {
        let requests_by_second = 30;
        let redis_prefix = "axum_rl_";
        let claims = Some(Err(app_error!(AppErrorCode::Unauthorized)));
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000);
        let addr = Some(ConnectInfo(socket));
        let mut key = "127.0.0.1".to_owned();
        key.insert_str(0, redis_prefix);

        assert_eq!(
            RateLimiterCheck::init(
                claims,
                addr.as_ref(),
                &vec!["127.0.0.1".to_owned()],
                redis_prefix,
                requests_by_second
            ),
            RateLimiterCheck {
                error: Some(RateLimiterError::JwtDecoding),
                key: None,
                limit: 0
            }
        );
    }
}
