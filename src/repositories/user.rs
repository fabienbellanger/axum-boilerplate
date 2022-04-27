use crate::models::user::{Login, User, UserCreation};
use chrono::{TimeZone, Utc};
use futures::stream::BoxStream;
use sha2::{Digest, Sha512};
use sqlx::mysql::MySqlRow;
use sqlx::{MySqlPool, Row};

pub struct UserRepository;

impl UserRepository {
    /// Returns a User if credentials are right
    #[instrument(skip_all, level = "warn")]
    pub async fn login(pool: &MySqlPool, input: Login) -> Result<Option<User>, sqlx::Error> {
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
                created_at: Utc.from_utc_datetime(&result.created_at),
                updated_at: Utc.from_utc_datetime(&result.updated_at),
                deleted_at: result.deleted_at.map(|d| Utc.from_utc_datetime(&d)),
            })),
            None => Ok(None),
        }
    }

    /// Add a new user
    #[tracing::instrument(skip(pool))]
    pub async fn create(pool: &MySqlPool, user: &mut User) -> Result<(), sqlx::Error> {
        user.password = format!("{:x}", Sha512::digest(user.password.as_bytes()));

        sqlx::query!(
            r#"
                INSERT INTO users (id, lastname, firstname, username, password, roles, created_at, updated_at, deleted_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            user.id,
            user.lastname,
            user.firstname,
            user.username,
            user.password,
            user.roles,
            user.created_at,
            user.updated_at,
            user.deleted_at,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Returns all users not deleted
    #[instrument(skip(pool))]
    pub fn get_all(pool: &MySqlPool) -> BoxStream<Result<Result<User, sqlx::Error>, sqlx::Error>> {
        sqlx::query(
            r#"
            SELECT id, username, password, lastname, firstname, roles, created_at, updated_at, deleted_at 
            FROM users 
            WHERE deleted_at IS NULL"#,
        )
        .map(|row: MySqlRow| {
            Ok(User {
                id: row.try_get(0)?,
                username: row.try_get(1)?,
                password: row.try_get(2)?,
                lastname: row.try_get(3)?,
                firstname: row.try_get(4)?,
                roles: row.try_get(5)?,
                created_at: row.try_get(6)?,
                updated_at: row.try_get(7)?,
                deleted_at: row.try_get(8)?,
            })
        })
        .fetch(pool)
    }

    /// Returns a user by its ID
    #[instrument(skip(pool))]
    pub async fn get_by_id(pool: &MySqlPool, id: String) -> Result<Option<User>, sqlx::Error> {
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
                created_at: Utc.from_utc_datetime(&result.created_at),
                updated_at: Utc.from_utc_datetime(&result.updated_at),
                deleted_at: result.deleted_at.map(|d| Utc.from_utc_datetime(&d)),
            })),
            None => Ok(None),
        }
    }

    /// Delete a user
    #[instrument(skip(pool))]
    pub async fn delete(pool: &MySqlPool, id: String) -> Result<u64, sqlx::Error> {
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
    #[instrument(skip(pool))]
    pub async fn update(pool: &MySqlPool, id: String, user: &UserCreation) -> Result<(), sqlx::Error> {
        let hashed_password = format!("{:x}", Sha512::digest(user.password.as_bytes()));
        sqlx::query!(
            r#"
                UPDATE users
                SET lastname = ?, firstname = ?, username = ?, password = ?, updated_at = ?
                WHERE id = ?
            "#,
            user.lastname,
            user.firstname,
            user.username,
            hashed_password,
            Some(Utc::now()),
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
