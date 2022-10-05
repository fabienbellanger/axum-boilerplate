//! Authentification module

use crate::errors::AppError;
use axum::http::{header, HeaderMap};
use chrono::Utc;
use color_eyre::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
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
    pub user_limit: i64,
}

impl Claims {
    /// Extract claims from request headers
    pub fn extract_from_request(headers: &HeaderMap, decoding_key: &DecodingKey) -> Option<Result<Self, AppError>> {
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
        roles: String,
        encoding_key: &EncodingKey,
        jwt_lifetime: i64,
    ) -> Result<(String, i64), AppError> {
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
            user_limit: 10, // TODO: From DB
        };

        let token = encode(&header, &payload, encoding_key).map_err(|err| {
            error!("error during JWT encoding: {err}");
            AppError::InternalError {
                message: "error during JWT encoding".to_string(),
            }
        })?;

        Ok((token, expired_at))
    }

    /// Parse JWT
    pub fn parse(token: &str, decoding_key: &DecodingKey) -> Result<Claims, AppError> {
        let validation = Validation::new(Algorithm::HS512);
        let token = decode::<Claims>(token, decoding_key, &validation).map_err(|err| {
            error!("error during JWT decoding: {err}");
            AppError::InternalError {
                message: "error during JWT decoding".to_string(),
            }
        })?;

        Ok(token.claims)
    }
}
