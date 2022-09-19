//! Helpers for user API tests

use super::TestResponse;
use crate::helper::{TestApp, TestDatabase};
use axum_boilerplate::{
    models::user::{LoginResponse, Role, User},
    repositories::user::UserRepository,
};
use chrono::Utc;
use uuid::Uuid;

/// Create a user for authentication
async fn create_user(db: &TestDatabase) -> User {
    let password = String::from("00000000");
    let mut user = User {
        id: Uuid::new_v4().to_string(),
        lastname: String::from("Doe"),
        firstname: String::from("John"),
        username: String::from("john.doe@test.com"),
        password: password.clone(),
        roles: Some(Role::User.to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let pool = db.database().await;
    UserRepository::create(&pool, &mut user)
        .await
        .expect("error during user creation");

    user.password = password;

    user
}

/// Create, authenticate a user and return `TestResponse` and the generated JWT
pub async fn create_and_authenticate(app: &TestApp) -> (TestResponse, String) {
    let db = app.database();
    let user = create_user(db).await;
    let response = login_request(
        &app,
        serde_json::json!({
            "username": user.username,
            "password": user.password
        })
        .to_string(),
    )
    .await;

    let res: LoginResponse = serde_json::from_str(&response.body.to_string()).expect("error when deserializing body");

    (response, res.token)
}

/// Login request helper
pub async fn login_request(app: &TestApp, body: String) -> TestResponse {
    TestResponse::new(app, "/api/v1/login", "POST", Some(body), None).await
}

/// User creation request helper
pub async fn create_user_request(app: &TestApp, body: String, token: &str) -> TestResponse {
    TestResponse::new(app, "/api/v1/users", "POST", Some(body), Some(token)).await
}

/// Return all users
pub async fn get_all(app: &TestApp, token: &str) -> TestResponse {
    TestResponse::new(app, "/api/v1/users", "GET", None, Some(token)).await
}

/// Return a user
pub async fn get_one(app: &TestApp, token: &str, id: &str) -> TestResponse {
    TestResponse::new(app, &format!("/api/v1/users/{id}"), "GET", None, Some(token)).await
}

/// Delete a user
pub async fn delete(app: &TestApp, token: &str, id: &str) -> TestResponse {
    TestResponse::new(app, &format!("/api/v1/users/{id}"), "DELETE", None, Some(token)).await
}

/// Update a user
pub async fn update(app: &TestApp, body: String, token: &str, id: &str) -> TestResponse {
    TestResponse::new(app, &format!("/api/v1/users/{id}"), "PUT", Some(body), Some(token)).await
}
