//! User model module

use chrono::Duration;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::Add,
};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub lastname: String,
    pub firstname: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub roles: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    /// Create a new `User` from `UserCreation`
    pub fn new(user: UserCreation) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            lastname: user.lastname,
            firstname: user.firstname,
            username: user.username,
            password: user.password,
            roles: user.roles,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct Login {
    #[validate(email)]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct LoginResponse {
    pub id: String,
    pub lastname: String,
    pub firstname: String,
    #[validate(email)]
    pub username: String,
    pub roles: String,
    pub token: String,
    pub expires_at: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UserCreation {
    pub lastname: String,
    pub firstname: String,
    #[validate(email)]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub roles: Option<String>,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserUpdatePassword {
    #[validate(length(min = 8))]
    pub password: String,
}

/// Defines user roles. Be carefull, roles are case sensitive (uppercase)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Role {
    User,
    Manager,
    Admin,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::User => "USER",
                Self::Manager => "MANAGER",
                Self::Admin => "ADMIN",
            }
        )
    }
}

impl Role {
    /// Try to return a `Role` if string role is valid
    /// TODO: Implement From trait instead!
    fn try_from_str(role: &str) -> Option<Self> {
        let mut roles = HashMap::with_capacity(3);
        roles.insert(format!("{}", Self::User), Self::User);
        roles.insert(format!("{}", Self::Manager), Self::Manager);
        roles.insert(format!("{}", Self::Admin), Self::Admin);

        roles.get(role).cloned()
    }

    /// Return `Role` list from string roles
    pub fn get_list(roles: &str) -> HashSet<Self> {
        roles
            .split(',')
            .map(|r| r.trim())
            .collect::<HashSet<&str>>()
            .iter()
            .filter_map(|r| Self::try_from_str(*r))
            .collect()
    }
}

/// Use to test if a password is strong enought
pub struct PasswordScorer {}

impl PasswordScorer {
    /// Valid that a password is strong enough (score >= 'good')
    ///
    /// A password whose score is:
    /// - 0 ~ 20 is very dangerous (may be cracked within few seconds)
    /// - 20 ~ 40 is dangerous
    /// - 40 ~ 60 is very weak
    /// - 60 ~ 80 is weak
    /// - 80 ~ 90 is good
    /// - 90 ~ 95 is strong
    /// - 95 ~ 99 is very strong
    /// - 99 ~ 100 is invulnerable
    pub fn valid(password: &str) -> bool {
        let score = passwords::scorer::score(&passwords::analyzer::analyze(password));
        score >= 80.0
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasswordReset {
    #[serde(skip_serializing)]
    pub user_id: String,
    pub token: String,
    pub expired_at: DateTime<Utc>,
}

impl PasswordReset {
    /// Create a new password recovery
    pub fn new(user_id: String, expiration_duration: i64) -> Self {
        let now = Utc::now();

        Self {
            user_id,
            token: Uuid::new_v4().to_string(),
            expired_at: now.add(Duration::hours(expiration_duration)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_try_from() {
        assert_eq!(Role::try_from_str("ADMIN"), Some(Role::Admin));
        assert_eq!(Role::try_from_str("MANAGER"), Some(Role::Manager));
        assert_eq!(Role::try_from_str("USER"), Some(Role::User));
        assert_eq!(Role::try_from_str("Admin"), None);
        assert_eq!(Role::try_from_str(""), None);
    }

    #[test]
    fn test_role_get_list() {
        let mut roles = HashSet::new();
        roles.insert(Role::Admin);
        assert_eq!(Role::get_list("ADMIN"), roles);

        let mut roles = HashSet::new();
        roles.insert(Role::Admin);
        roles.insert(Role::User);
        assert_eq!(Role::get_list("ADMIN,USER"), roles);
        assert_eq!(Role::get_list("USER,ADMIN"), roles);
        assert_eq!(Role::get_list(" USER , ADMIN"), roles);

        assert_eq!(Role::get_list(""), HashSet::new());
        assert_eq!(Role::get_list(" "), HashSet::new());
    }

    #[test]
    fn test_passwords_score() {
        // Not valid
        assert!(!PasswordScorer::valid(""));
        assert!(!PasswordScorer::valid("azerty"));
        assert!(!PasswordScorer::valid("azerty"));

        // Valid
        dbg!(passwords::scorer::score(&passwords::analyzer::analyze("Ad15,df7js")));
        assert!(PasswordScorer::valid("Ad15,df7js"));
    }
}
