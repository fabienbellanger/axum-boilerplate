use crate::app_error;
use crate::errors::{AppError, AppErrorCode};
use crate::models::user::{Login, PasswordReset, User, UserCreation};
use crate::utils::query::PaginateSort;
use chrono::{TimeZone, Utc};
use futures::TryStreamExt;
use sha2::{Digest, Sha512};
use sqlx::{MySqlPool, Row};

pub struct UserRepository;

impl UserRepository {
    /// Returns a User if credentials are right
    #[instrument(skip_all, level = "warn")]
    pub async fn login(pool: &MySqlPool, input: Login) -> Result<Option<User>, AppError> {
        warn!("LOGIN repo");
        let hashed_password = format!("{:x}", Sha512::digest(input.password.as_bytes()));
        let result = sqlx::query!(
            r#"
                SELECT * 
                FROM users 
                WHERE username = ?
                    AND password = ?
                    AND deleted_at IS NULL
            "#,
            input.username,
            hashed_password
        )
        .fetch_optional(pool)
        .await?;

        match result {
            Some(result) => Ok(Some(User {
                id: result.id,
                lastname: result.lastname,
                firstname: result.firstname,
                username: result.username,
                password: result.password,
                roles: result.roles,
                rate_limit: result.rate_limit,
                created_at: Utc.from_utc_datetime(&result.created_at),
                updated_at: Utc.from_utc_datetime(&result.updated_at),
                deleted_at: result.deleted_at.map(|d| Utc.from_utc_datetime(&d)),
            })),
            None => Ok(None),
        }
    }

    /// Add a new user
    #[tracing::instrument(skip(pool))]
    pub async fn create(pool: &MySqlPool, user: &mut User) -> Result<(), AppError> {
        user.password = format!("{:x}", Sha512::digest(user.password.as_bytes()));

        sqlx::query!(
            r#"
                INSERT INTO `users` (`id`, `lastname`, `firstname`, `username`, `password`, `roles`, `rate_limit`, `created_at`, `updated_at`, `deleted_at`)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            user.id,
            user.lastname,
            user.firstname,
            user.username,
            user.password,
            user.roles,
            user.rate_limit,
            user.created_at,
            user.updated_at,
            user.deleted_at,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Returns all not deleted users
    #[instrument(skip(pool))]
    pub async fn get_all<'a>(pool: &'a MySqlPool, paginate_sort: &'a PaginateSort) -> Result<Vec<User>, AppError> {
        let mut query = String::from(
            "
            SELECT id, username, password, lastname, firstname, roles, rate_limit, created_at, updated_at, deleted_at 
            FROM users 
            WHERE deleted_at IS NULL
            ",
        );

        // Sorts and pagination
        query.push_str(&paginate_sort.get_sorts_sql(Some(vec![
            "id",
            "lastname",
            "firstname",
            "created_at",
            "updated_at",
            "deleted_at",
        ])));
        query.push_str(&paginate_sort.get_pagination_sql());

        let mut rows = sqlx::query(&query)
            .bind(paginate_sort.limit)
            .bind(paginate_sort.offset)
            .fetch(pool);

        let mut users = vec![];
        while let Some(row) = rows.try_next().await? {
            users.push(User {
                id: row.try_get("id")?,
                lastname: row.try_get("lastname")?,
                firstname: row.try_get("firstname")?,
                username: row.try_get("username")?,
                password: row.try_get("password")?,
                roles: row.try_get("roles")?,
                rate_limit: row.try_get("rate_limit")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                deleted_at: row.try_get("deleted_at")?,
            });
        }
        Ok(users)
    }

    /// Returns a user by its ID
    #[instrument(skip(pool))]
    pub async fn get_by_id(pool: &MySqlPool, id: String) -> Result<Option<User>, AppError> {
        let result = sqlx::query!(
            r#"
                SELECT * 
                FROM users 
                WHERE id = ?
                    AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        match result {
            Some(result) => Ok(Some(User {
                id: result.id,
                lastname: result.lastname,
                firstname: result.firstname,
                username: result.username,
                password: result.password,
                roles: result.roles,
                rate_limit: result.rate_limit,
                created_at: Utc.from_utc_datetime(&result.created_at),
                updated_at: Utc.from_utc_datetime(&result.updated_at),
                deleted_at: result.deleted_at.map(|d| Utc.from_utc_datetime(&d)),
            })),
            None => Ok(None),
        }
    }

    /// Returns a user by its email
    #[instrument(skip(pool))]
    pub async fn get_by_email(pool: &MySqlPool, email: String) -> Result<Option<User>, AppError> {
        let result = sqlx::query!(
            r#"
                SELECT * 
                FROM users 
                WHERE username = ?
                    AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(pool)
        .await?;

        match result {
            Some(result) => Ok(Some(User {
                id: result.id,
                lastname: result.lastname,
                firstname: result.firstname,
                username: result.username,
                password: result.password,
                roles: result.roles,
                rate_limit: result.rate_limit,
                created_at: Utc.from_utc_datetime(&result.created_at),
                updated_at: Utc.from_utc_datetime(&result.updated_at),
                deleted_at: result.deleted_at.map(|d| Utc.from_utc_datetime(&d)),
            })),
            None => Ok(None),
        }
    }

    /// Delete a user
    #[instrument(skip(pool))]
    pub async fn delete(pool: &MySqlPool, id: String) -> Result<u64, AppError> {
        let result = sqlx::query!(
            r#"
                UPDATE users
                SET deleted_at = ?
                WHERE id = ? AND deleted_at IS NULL
            "#,
            Some(Utc::now()),
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Update a user
    // TODO: Check if roles, rate_limit, etc. are valid
    #[instrument(skip(pool))]
    pub async fn update(pool: &MySqlPool, id: String, user: &UserCreation) -> Result<(), AppError> {
        let hashed_password = format!("{:x}", Sha512::digest(user.password.as_bytes()));
        sqlx::query!(
            r#"
                UPDATE users
                SET lastname = ?, firstname = ?, username = ?, password = ?, roles = ?, rate_limit = ?, updated_at = ?
                WHERE id = ?
            "#,
            user.lastname,
            user.firstname,
            user.username,
            hashed_password,
            user.roles,
            user.rate_limit,
            Some(Utc::now()),
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(pool))]
    pub async fn update_password(
        pool: &MySqlPool,
        id: String,
        current_password: String,
        new_password: String,
    ) -> Result<(), AppError> {
        let hashed_password = format!("{:x}", Sha512::digest(new_password.as_bytes()));

        if hashed_password == current_password {
            return Err(app_error!(
                AppErrorCode::BadRequest,
                "new password cannot be the same as the current one"
            ));
        }

        sqlx::query!(
            r#"
                UPDATE users
                SET password = ?, updated_at = ?
                WHERE id = ?
            "#,
            hashed_password,
            Some(Utc::now()),
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub struct PasswordResetRepository;

impl PasswordResetRepository {
    /// Add a new password reset
    #[tracing::instrument(skip(pool))]
    pub async fn create_or_update(pool: &MySqlPool, password_reset: &mut PasswordReset) -> Result<(), AppError> {
        sqlx::query!(
            r#"
                INSERT INTO password_resets (user_id, token, expired_at)
                VALUES (?, ?, ?)
                ON DUPLICATE KEY UPDATE token = ?, expired_at = ?
            "#,
            password_reset.user_id,
            password_reset.token,
            password_reset.expired_at,
            password_reset.token,
            password_reset.expired_at,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get user ID from token
    #[tracing::instrument(skip(pool))]
    pub async fn get_user_id_from_token(pool: &MySqlPool, token: String) -> Result<Option<(String, String)>, AppError> {
        let result = sqlx::query!(
            r#"
                SELECT u.id AS user_id, u.password AS password
                FROM password_resets pr
                    INNER JOIN users u ON u.id = pr.user_id AND u.deleted_at IS NULL
                WHERE pr.token = ?
                    AND pr.expired_at >= ?
            "#,
            token,
            Utc::now(),
        )
        .fetch_optional(pool)
        .await?;

        match result {
            Some(result) => Ok(Some((result.user_id, result.password))),
            None => Ok(None),
        }
    }

    /// Delete password reset after successfull update
    #[tracing::instrument(skip(pool))]
    pub async fn delete(pool: &MySqlPool, user_id: String) -> Result<u64, AppError> {
        let result = sqlx::query!(
            r#"
                DELETE FROM password_resets
                WHERE user_id = ?
            "#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
