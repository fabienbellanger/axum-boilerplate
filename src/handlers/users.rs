//! API users handlers

use crate::models::auth::Jwt;
use crate::models::user::{Login, User, UserCreation};
use crate::repositories::user::UserRepository;
use crate::utils::extractors::ExtractRequestId;
use crate::utils::validation::validate_request_data;
use crate::{
    errors::{AppError, AppResult},
    layers::SharedState,
    models::user::LoginResponse,
};
use axum::extract::{Extension, Json, Path};
use axum::http::StatusCode;
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use futures::TryStreamExt;
use sqlx::{MySql, Pool};
use uuid::Uuid;

// Route: POST /api/v1/login
#[instrument(name = "Login", skip(pool, state), level = "warn")]
pub async fn login(
    Json(payload): Json<Login>,
    Extension(pool): Extension<Pool<MySql>>,
    Extension(state): Extension<SharedState>,
    ExtractRequestId(request_id): ExtractRequestId,
) -> AppResult<Json<LoginResponse>> {
    warn!("LOGIN handler");
    validate_request_data(&payload)?;

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
#[instrument(skip(pool))]
pub async fn create(
    Json(payload): Json<UserCreation>,
    Extension(pool): Extension<Pool<MySql>>,
) -> AppResult<Json<User>> {
    validate_request_data(&payload)?;

    let mut user = User::new(payload);
    UserRepository::create(&pool, &mut user).await?;

    Ok(Json(user))
}

// Route: GET /api/v1/users
#[instrument(skip(pool))]
pub async fn get_all(Extension(pool): Extension<Pool<MySql>>) -> AppResult<Json<Vec<User>>> {
    let mut stream = UserRepository::get_all(&pool);
    let mut users: Vec<User> = Vec::new();
    while let Some(row) = stream.try_next().await? {
        users.push(row?);
    }

    Ok(Json(users))
}

// Route: GET "/v1/users/:id"
#[instrument(skip(pool))]
pub async fn get_by_id(Path(id): Path<Uuid>, Extension(pool): Extension<Pool<MySql>>) -> AppResult<Json<User>> {
    let user = UserRepository::get_by_id(&pool, id.to_string()).await?;
    match user {
        Some(user) => Ok(Json(user)),
        _ => Err(AppError::NotFound {
            message: String::from("No user found"),
        }),
    }
}

// Route: DELETE "/v1/users/:id"
#[instrument(skip(pool))]
pub async fn delete(Path(id): Path<Uuid>, Extension(pool): Extension<Pool<MySql>>) -> AppResult<StatusCode> {
    let result = UserRepository::delete(&pool, id.to_string()).await?;
    match result {
        1 => Ok(StatusCode::NO_CONTENT),
        _ => Err(AppError::InternalError {
            message: String::from("No user or user already deleted"),
        }),
    }
}

// Route: PUT "/v1/users/:id"
#[instrument(skip(pool))]
pub async fn update(
    Path(id): Path<Uuid>,
    Json(payload): Json<UserCreation>,
    Extension(pool): Extension<Pool<MySql>>,
) -> AppResult<Json<User>> {
    UserRepository::update(&pool, id.to_string(), &payload).await?;

    let user = UserRepository::get_by_id(&pool, id.to_string()).await?;
    match user {
        Some(user) => Ok(Json(user)),
        _ => Err(AppError::NotFound {
            message: String::from("No user found"),
        }),
    }
}
