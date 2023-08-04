//! Authentification module

use crate::{
    app_error,
    utils::errors::{AppError, AppErrorCode, AppResult},
};
use axum::http::{header, HeaderMap};
use chrono::Utc;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind::ExpiredSignature, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub user_id: String,
    pub user_roles: String,

    /// Max number of request by second (-1: unlimited)
    pub user_rate_limit: i32,
}

impl Claims {
    /// Extract claims from request headers
    pub fn extract_from_request(headers: &HeaderMap, decoding_key: &DecodingKey) -> Option<AppResult<Self>> {
        headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                let words = h.split("Bearer").collect::<Vec<&str>>();
                words.get(1).map(|w| w.trim())
            })
            .map(|token| Jwt::parse(token, decoding_key))
    }
}

pub struct Jwt {}

impl Jwt {
    /// Generate JWT
    pub fn generate(
        user_id: String,
        user_rate_limit: i32,
        roles: String,
        encoding_key: &EncodingKey,
        jwt_lifetime: i64,
    ) -> AppResult<(String, i64)> {
        let header = Header::new(Algorithm::HS512);
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nanosecond -> second
        let expired_at = now + (jwt_lifetime * 3600);

        let payload = Claims {
            sub: user_id.clone(),
            exp: expired_at,
            iat: now,
            nbf: now,
            user_id,
            user_roles: roles,
            user_rate_limit,
        };

        let token = encode(&header, &payload, encoding_key).map_err(|err| {
            app_error!(
                AppErrorCode::InternalError,
                "error during JWT encoding",
                format!("error during JWT encoding: {err}")
            )
        })?;

        Ok((token, expired_at))
    }

    /// Parse JWT
    pub fn parse(token: &str, decoding_key: &DecodingKey) -> AppResult<Claims> {
        let validation = Validation::new(Algorithm::HS512);
        let token = decode::<Claims>(token, decoding_key, &validation).map_err(|err| match err.kind() {
            ExpiredSignature => app_error!(
                AppErrorCode::InternalError,
                "error during JWT decoding",
                format!("error during JWT decoding: {err}")
            ),
            _ => app_error!(AppErrorCode::InternalError),
        })?;

        Ok(token.claims)
    }
}
