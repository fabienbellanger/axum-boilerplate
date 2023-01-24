//! API users handlers

use crate::{
    app_error,
    emails::{forgotten_password::ForgottenPasswordEmail, SmtpConfig},
    layers::SharedState,
    models::{
        auth::Jwt,
        user::{Login, LoginResponse, PasswordReset, User, UserCreation, UserUpdatePassword},
    },
    repositories::user::{PasswordResetRepository, UserRepository},
    utils::{
        errors::{AppError, AppErrorCode, AppResult},
        extractors::{ExtractRequestId, Path},
        query::{PaginateResponse, PaginateSort, PaginateSortQuery},
        validation::validate_request_data,
    },
};
use axum::{
    extract::{Extension, Json, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use sqlx::{MySql, Pool};
use uuid::Uuid;

// Route: POST /api/v1/login
#[instrument(name = "Login handler", skip(pool, state), level = "warn")]
pub async fn login(
    Extension(pool): Extension<Pool<MySql>>,
    State(state): State<SharedState>,
    ExtractRequestId(request_id): ExtractRequestId,
    Json(payload): Json<Login>,
) -> AppResult<Json<LoginResponse>> {
    warn!("In Login handler");

    validate_request_data(&payload)?;

    // Search user in database and return `LoginResponse`
    let user = UserRepository::login(&pool, payload).await?;
    match user {
        None => Err(app_error!(AppErrorCode::Unauthorized)),
        Some(user) => {
            // Token generation
            let encoding_key = &state.config.jwt_encoding_key.clone();
            let jwt_lifetime = state.config.jwt_lifetime;
            let roles = match user.roles {
                Some(roles) => roles,
                None => String::new(),
            };
            let token = Jwt::generate(
                user.id.to_owned(),
                user.rate_limit,
                roles.clone(),
                encoding_key,
                jwt_lifetime,
            );

            match token {
                Ok(token) => match NaiveDateTime::from_timestamp_opt(token.1, 0) {
                    Some(expires_at) => {
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
                    None => Err(app_error!(
                        AppErrorCode::InternalError,
                        "error during JWT generation",
                        format!(
                            "error during JWT generation: invalid 'expired_at' field in JWT claims ({})",
                            token.1
                        )
                    )),
                },
                Err(err) => Err(app_error!(
                    AppErrorCode::InternalError,
                    "error during JWT generation",
                    format!("error during JWT generation: {err}")
                )),
            }
        }
    }
}

// Route: POST /api/v1/users
#[instrument(skip(pool))]
pub async fn create(
    Extension(pool): Extension<Pool<MySql>>,
    ExtractRequestId(request_id): ExtractRequestId,
    Json(payload): Json<UserCreation>,
) -> AppResult<Json<User>> {
    validate_request_data(&payload)?;

    let mut user = User::new(payload);
    UserRepository::create(&pool, &mut user).await?;

    Ok(Json(user))
}

// Route: GET /api/v1/users
// TODO: Add filter
#[instrument(skip(pool))]
pub async fn get_all(
    Query(pagination): Query<PaginateSortQuery>,
    Extension(pool): Extension<Pool<MySql>>,
    ExtractRequestId(request_id): ExtractRequestId,
) -> AppResult<Json<PaginateResponse<Vec<User>>>> {
    let paginate_sort = PaginateSort::from(pagination);
    let users = UserRepository::get_all(&pool, &paginate_sort).await?;

    Ok(Json(users))
}

// Route: GET "/api/v1/users/:id"
#[instrument(skip(pool))]
pub async fn get_by_id(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<Pool<MySql>>,
    ExtractRequestId(request_id): ExtractRequestId,
) -> AppResult<Json<User>> {
    let user = UserRepository::get_by_id(&pool, id.to_string()).await?;
    match user {
        Some(user) => Ok(Json(user)),
        _ => Err(app_error!(AppErrorCode::NotFound, "no user found")),
    }
}

// Route: DELETE "/api/v1/users/:id"
#[instrument(skip(pool))]
pub async fn delete(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<Pool<MySql>>,
    ExtractRequestId(request_id): ExtractRequestId,
) -> AppResult<StatusCode> {
    let result = UserRepository::delete(&pool, id.to_string()).await?;
    match result {
        1 => Ok(StatusCode::NO_CONTENT),
        _ => Err(app_error!(
            AppErrorCode::InternalError,
            "no user or user already deleted"
        )),
    }
}

// Route: PUT "/api/v1/users/:id"
#[instrument(skip(pool))]
pub async fn update(
    Path(id): Path<Uuid>,
    Extension(pool): Extension<Pool<MySql>>,
    ExtractRequestId(request_id): ExtractRequestId,
    Json(payload): Json<UserCreation>,
) -> AppResult<Json<User>> {
    validate_request_data(&payload)?;

    UserRepository::update(&pool, id.to_string(), &payload).await?;

    let user = UserRepository::get_by_id(&pool, id.to_string()).await?;
    match user {
        Some(user) => Ok(Json(user)),
        _ => Err(app_error!(AppErrorCode::NotFound, "no user found")),
    }
}

// Route: POST "/api/v1/forgotten-password/:email"
#[instrument(skip(pool, state))]
pub async fn forgotten_password(
    Path(email): Path<String>,
    State(state): State<SharedState>,
    Extension(pool): Extension<Pool<MySql>>,
    ExtractRequestId(request_id): ExtractRequestId,
) -> AppResult<Json<PasswordReset>> {
    match UserRepository::get_by_email(&pool, email.clone()).await? {
        None => Err(app_error!(AppErrorCode::NotFound, "no user found")),
        Some(user) => {
            let mut password_reset = PasswordReset::new(user.id, state.config.forgotten_password_expiration_duration);

            // Save in database
            PasswordResetRepository::create_or_update(&pool, &mut password_reset).await?;

            // Send email
            ForgottenPasswordEmail::send(
                &SmtpConfig {
                    host: state.config.smtp_host.clone(),
                    port: state.config.smtp_port,
                    timeout: state.config.smtp_timeout,
                },
                state.config.forgotten_password_base_url.clone(),
                state.config.forgotten_password_email_from.clone(),
                email,
                password_reset.token.clone(),
            )?;

            Ok(Json(password_reset))
        }
    }
}

// Route: PATCH "/api/v1/update-password/:token"
#[instrument(skip(pool))]
pub async fn update_password(
    Path(token): Path<Uuid>,
    Extension(pool): Extension<Pool<MySql>>,
    ExtractRequestId(request_id): ExtractRequestId,
    Json(payload): Json<UserUpdatePassword>,
) -> AppResult<StatusCode> {
    validate_request_data(&payload)?;

    let result = PasswordResetRepository::get_user_id_from_token(&pool, token.to_string()).await?;
    match result {
        Some((user_id, current_password)) => {
            // Update user password
            let password = payload.password;
            UserRepository::update_password(&pool, user_id.clone(), current_password, password).await?;

            // Delete password reset entry
            PasswordResetRepository::delete(&pool, user_id).await?;

            Ok(StatusCode::OK)
        }
        _ => Err(app_error!(AppErrorCode::NotFound, "no user found")),
    }
}
