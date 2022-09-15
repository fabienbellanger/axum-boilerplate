//! Helpers for user API tests

use super::TestResponse;
use crate::helper::{TestApp, TestDatabase};
use axum::http::Request;
use axum_boilerplate::{
    models::user::{LoginResponse, Role, User},
    repositories::user::UserRepository,
};
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use tower::ServiceExt;
use uuid::Uuid;

/// Login request helper
pub async fn login_request(app: &TestApp, body: String) -> TestResponse {
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(body.into())
                .unwrap(),
        )
        .await
        .unwrap();

    let status_code = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();

    TestResponse {
        status_code,
        body,
        headers: HashMap::new(),
    }
}

/// User creation helper
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

    let res: LoginResponse = serde_json::from_str(&response.body.to_string()).expect("unable to deserialize body");

    (response, res.token)
}

/// User creation request helper
pub async fn create_user_request(app: &TestApp, body: String, token: String) -> TestResponse {
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(body.into())
                .unwrap(),
        )
        .await
        .unwrap();

    let status_code = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();

    TestResponse {
        status_code,
        body,
        headers: HashMap::new(),
    }
}
