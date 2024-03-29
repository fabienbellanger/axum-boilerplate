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
    pub rate_limit: i32,
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
            rate_limit: user.rate_limit,
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
    pub rate_limit: i32,
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
            .filter_map(|r| Self::try_from_str(r))
            .collect()
    }
}

/// Password strength
pub enum PasswordStrength {
    Dangerous,    // 0..40
    VeryWeak,     // 40..60
    Weak,         // 60..80
    Good,         // 80..90
    Strong,       // 90..95
    VeryStrong,   // 95..99
    Invulnerable, // 99..100
}

const PASSWORD_STRENGTH_DANGEROUS: f64 = 0.0;
const PASSWORD_STRENGTH_VERY_WEAK: f64 = 40.0;
const PASSWORD_STRENGTH_WEAK: f64 = 60.0;
const PASSWORD_STRENGTH_GOOD: f64 = 80.0;
const PASSWORD_STRENGTH_STRONG: f64 = 90.0;
const PASSWORD_STRENGTH_VERY_STRONG: f64 = 95.0;
const PASSWORD_STRENGTH_INVULNERABLE: f64 = 99.0;

/// Used to test if a password is strong enought
pub struct PasswordScorer {}

impl PasswordScorer {
    /// Valid that a password is strong enough
    pub fn valid(password: &str, strength: PasswordStrength) -> bool {
        let score = passwords::scorer::score(&passwords::analyzer::analyze(password));
        match strength {
            PasswordStrength::Dangerous => PASSWORD_STRENGTH_DANGEROUS <= score,
            PasswordStrength::VeryWeak => PASSWORD_STRENGTH_VERY_WEAK <= score,
            PasswordStrength::Weak => PASSWORD_STRENGTH_WEAK <= score,
            PasswordStrength::Good => PASSWORD_STRENGTH_GOOD <= score,
            PasswordStrength::Strong => PASSWORD_STRENGTH_STRONG <= score,
            PasswordStrength::VeryStrong => PASSWORD_STRENGTH_VERY_STRONG <= score,
            PasswordStrength::Invulnerable => PASSWORD_STRENGTH_INVULNERABLE <= score,
        }
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
        assert!(!PasswordScorer::valid("", PasswordStrength::Strong));
        assert!(!PasswordScorer::valid("azerty", PasswordStrength::Strong));
        assert!(!PasswordScorer::valid("azerty", PasswordStrength::Strong));

        // Valid
        assert!(PasswordScorer::valid("", PasswordStrength::Dangerous));
        assert!(PasswordScorer::valid("azerty", PasswordStrength::Dangerous));
        assert!(PasswordScorer::valid("Wl6,Ak4;6a", PasswordStrength::Good));
        assert!(PasswordScorer::valid(
            "WlH5Y;8!fs81#6,Ak4;6a(HJ27hgh6g=1",
            PasswordStrength::Invulnerable
        ));
        assert!(PasswordScorer::valid("Wl6,Ak4;6a", PasswordStrength::Dangerous));
    }
}
