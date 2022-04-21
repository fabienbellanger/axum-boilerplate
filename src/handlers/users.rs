//! API users handlers

use crate::models::auth::Jwt;
use crate::models::user::{Login, User, UserCreation};
use crate::repositories::user::UserRepository;
use crate::{errors::AppError, layers::SharedState, models::user::LoginResponse};
use axum::http::StatusCode;
use axum::{extract::Path, Extension, Json};
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use futures::TryStreamExt;
use sqlx::{MySql, Pool};
use uuid::Uuid;
use validator::Validate;

// Route: POST /api/v1/login
#[tracing::instrument(name = "Login", skip(pool, state), fields())]
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

// Route: POST /api/v1/users
pub async fn create(
    Json(payload): Json<UserCreation>,
    Extension(pool): Extension<Pool<MySql>>,
) -> Result<Json<User>, AppError> {
    let payload_validation = payload.validate();
    if payload_validation.is_err() {
        error!("{}", payload_validation.clone().err().unwrap());
        payload_validation.err().map(|err| error!("{}", err));
    }

    let mut user = User::new(payload);
    UserRepository::create(&pool, &mut user).await?;

    Ok(Json(user))
}

// Route: GET /api/v1/users
pub async fn get_all(Extension(pool): Extension<Pool<MySql>>) -> Result<Json<Vec<User>>, AppError> {
    let mut stream = UserRepository::get_all(&pool);
    let mut users: Vec<User> = Vec::new();
    while let Some(row) = stream.try_next().await? {
        users.push(row?);
    }

    Ok(Json(users))
}

// Route: GET "/v1/users/:id"
pub async fn get_by_id(Path(id): Path<Uuid>, Extension(pool): Extension<Pool<MySql>>) -> Result<Json<User>, AppError> {
    let user = UserRepository::get_by_id(&pool, id.to_string()).await?;
    match user {
        Some(user) => Ok(Json(user)),
        _ => Err(AppError::NotFound {
            message: String::from("No user found"),
        }),
    }
}

// Route: DELETE "/v1/users/:id"
pub async fn delete(Path(id): Path<Uuid>, Extension(pool): Extension<Pool<MySql>>) -> Result<StatusCode, AppError> {
    let result = UserRepository::delete(&pool, id.to_string()).await?;
    match result {
        1 => Ok(StatusCode::NO_CONTENT),
        _ => Err(AppError::InternalError {
            message: String::from("No user or user already deleted"),
        }),
    }
}

// Route: PUT "/v1/users/:id"
pub async fn update(
    Path(id): Path<Uuid>,
    Json(payload): Json<UserCreation>,
    Extension(pool): Extension<Pool<MySql>>,
) -> Result<Json<User>, AppError> {
    UserRepository::update(&pool, id.to_string(), &payload).await?;

    let user = UserRepository::get_by_id(&pool, id.to_string()).await?;
    match user {
        Some(user) => Ok(Json(user)),
        _ => Err(AppError::NotFound {
            message: String::from("No user found"),
        }),
    }
}
