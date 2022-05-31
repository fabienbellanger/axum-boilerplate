//! Authentification module

use axum::http::{header, HeaderMap};
use chrono::Utc;
use color_eyre::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// DOC: https://github.com/tokio-rs/axum/blob/main/examples/jwt/src/main.rs

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
    pub fn extract_from_request(headers: &HeaderMap, secret_key: String) -> Option<Self> {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                let words = h.split("Bearer").collect::<Vec<&str>>();
                words.get(1).map(|w| w.trim())
            });

        match token {
            Some(token) => Jwt::parse(token, secret_key).ok(),
            None => None,
        }
    }

    /// Extract claims from token
    pub fn from_token(token: &str, secret_key: String) -> Option<Self> {
        Jwt::parse(token, secret_key).ok()
    }
}

pub struct Jwt {}

impl Jwt {
    /// Generate JWT
    /// TODO: Params in a struct
    /// TODO: Use custom error instead of Box<dyn std::error::Error>
    pub fn generate(
        user_id: String,
        roles: String,
        secret_key: String,
        jwt_lifetime: i64,
    ) -> Result<(String, i64), Box<dyn std::error::Error>> {
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

        let token = encode(&header, &payload, &EncodingKey::from_secret(secret_key.as_bytes()))?;

        Ok((token, expired_at))
    }

    /// Parse JWT
    /// TODO: Use custom error instead of Box<dyn std::error::Error>
    pub fn parse(token: &str, secret_key: String) -> Result<Claims, Box<dyn std::error::Error>> {
        let validation = Validation::new(Algorithm::HS512);
        let token = decode::<Claims>(token, &DecodingKey::from_secret(secret_key.as_bytes()), &validation)?;

        Ok(token.claims)
    }
}
