//! User model module

use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
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

#[derive(Serialize, Debug, Validate)]
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
}
