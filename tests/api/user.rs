use super::helpers::user::{create_and_authenticate, create_user_request, delete, get_all, get_one, login_request};
use super::helpers::TestUser;
use crate::helper::{TestApp, TestAppBuilder};
use axum::http::StatusCode;

#[tokio::test]
async fn test_api_login_unauthorized_user() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();

    let response = login_request(
        &app,
        serde_json::json!({
            "username": "test@gmail.com",
            "password": "00000000"
        })
        .to_string(),
    )
    .await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.body,
        serde_json::json!({
            "code": 401,
            "message": "Unauthorized"
        })
    );
}

#[tokio::test]
async fn test_api_login_authorized_user() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (response, _token) = create_and_authenticate(&app).await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::OK);
}

#[tokio::test]
async fn test_api_user_creation_success() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    let response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test",
            "firstname": "Toto"
        })
        .to_string(),
        &token,
    )
    .await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::OK);
}

#[tokio::test]
async fn test_api_user_creation_invalid_password() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    let response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "0000000",
            "lastname": "Test",
            "firstname": "Toto"
        })
        .to_string(),
        &token,
    )
    .await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_user_list_all() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    // Create 2 users
    for i in 1..3 {
        create_user_request(
            &app,
            serde_json::json!({
                "username": format!("test-user-creation-{i}@test.com"),
                "password": "00000000",
                "lastname": "Test",
                "firstname": format!("Toto {i}"),
            })
            .to_string(),
            &token,
        )
        .await;
    }

    let response = get_all(&app, &token).await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::OK);

    let users: Vec<TestUser> = serde_json::from_str(&response.body.to_string()).expect("error when deserializing body");
    assert_eq!(users.len(), 3);
}

#[tokio::test]
async fn test_api_user_list_one() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    // Create a user
    let response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test",
            "firstname": "Toto",
        })
        .to_string(),
        &token,
    )
    .await;

    // Get user ID
    let user: TestUser = serde_json::from_str(&response.body.to_string()).expect("error when deserializing body");
    let user_id = user.id;

    // Get one user by its ID
    let response = get_one(&app, &token, &user_id).await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::OK);

    let user: TestUser = serde_json::from_str(&response.body.to_string()).expect("error when deserializing body");
    assert_eq!(user.id, user_id);
}

#[tokio::test]
async fn test_api_user_get_one_bad_parameter() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    // Create a user
    let response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test",
            "firstname": "Toto",
        })
        .to_string(),
        &token,
    )
    .await;

    // Get user ID
    let _user: TestUser = serde_json::from_str(&response.body.to_string()).expect("error when deserializing body");

    // Get one user by its ID
    let response = get_one(&app, &token, "bad_id").await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_api_user_delete() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    // Create a user
    let response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test",
            "firstname": "Toto",
        })
        .to_string(),
        &token,
    )
    .await;

    // Get user ID
    let user: TestUser = serde_json::from_str(&response.body.to_string()).expect("error when deserializing body");
    let user_id = user.id;

    // Get one user by its ID
    let response = delete(&app, &token, &user_id).await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::NO_CONTENT);
}
