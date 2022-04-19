//! API users handlers

use crate::models::auth::Jwt;
use crate::models::user::Login;
use crate::repositories::user::UserRepository;
use crate::{errors::AppError, layers::SharedState, models::user::LoginResponse};
use axum::{Extension, Json};
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use sqlx::{MySql, Pool};
use validator::Validate;

// Route: POST /api/v1/login
pub async fn login(
    Json(payload): Json<Login>,
    Extension(pool): Extension<Pool<MySql>>,
    Extension(state): Extension<SharedState>,
) -> Result<Json<LoginResponse>, AppError> {
    // Payload validation
    // TODO: Return error with validator message
    let payload_validation = payload.validate();
    if payload_validation.is_err() {
        error!("{}", payload_validation.clone().err().unwrap());
        payload_validation.err().map(|err| error!("{}", err));
    }

    // Search user in database and return `LoginResponse`
    let user = UserRepository::login(&pool, payload).await?;
    match user {
        None => Err(AppError::Unauthorized {}),
        Some(user) => {
            // Token generation
            let secret = state.jwt_secret_key.clone();
            let jwt_lifetime = state.jwt_lifetime;
            let roles = match user.roles {
                Some(roles) => roles,
                None => String::new(),
            };
            let token = Jwt::generate(
                user.id.to_owned(),
                user.lastname.to_owned(),
                user.firstname.to_owned(),
                user.username.to_owned(),
                roles.clone(),
                secret,
                jwt_lifetime,
            );

            match token {
                Ok(token) => {
                    let expires_at = NaiveDateTime::from_timestamp(token.1, 0);
                    let expires_at: DateTime<Utc> = DateTime::from_utc(expires_at, Utc);

                    Ok(Json(LoginResponse {
                        id: user.id.to_owned(),
                        lastname: user.lastname.to_owned(),
                        firstname: user.firstname.to_owned(),
                        username: user.username,
                        roles,
                        token: token.0,
                        expires_at: expires_at.to_rfc3339_opts(SecondsFormat::Secs, true),
                    }))
                }
                _ => Err(AppError::InternalError {
                    message: String::from("error during JWT generation"),
                }),
            }
        }
    }
}

// Route: POST /api/v1/register
pub async fn register() -> Result<String, AppError> {
    Ok(String::from("Register route"))
}
