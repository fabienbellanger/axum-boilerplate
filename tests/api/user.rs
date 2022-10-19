use super::helpers::user::{
    create_and_authenticate, create_user_request, delete, forgotten_password, get_all, get_one,
    is_password_reset_token_still_in_database, login_request, update, update_password,
};
use super::helpers::TestUser;
use crate::api::helpers::TestPasswordReset;
use crate::helper::{TestApp, TestAppBuilder};
use axum::http::StatusCode;
use uuid::Uuid;

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
            "firstname": "Toto",
            "rate_limit": 10,
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
            "firstname": "Toto",
            "rate_limit": 10,
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
                "rate_limit": 10,
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
            "rate_limit": 10,
        })
        .to_string(),
        &token,
    )
    .await;

    // Get user ID
    let user_id = TestUser::from_body(&response.body.to_string()).id;

    // Get one user by its ID
    let response = get_one(&app, &token, &user_id).await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::OK);
    assert_eq!(TestUser::from_body(&response.body.to_string()).id, user_id);
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
            "rate_limit": 10,
        })
        .to_string(),
        &token,
    )
    .await;

    // Get user ID
    let _user = TestUser::from_body(&response.body.to_string());

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
            "rate_limit": 10,
        })
        .to_string(),
        &token,
    )
    .await;

    // Get user ID
    let user_id = TestUser::from_body(&response.body.to_string()).id;

    // Get one user by its ID
    let response = delete(&app, &token, &user_id).await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_api_user_update() {
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
            "rate_limit": 10,
        })
        .to_string(),
        &token,
    )
    .await;

    // Get user ID
    let user_id = TestUser::from_body(&response.body.to_string()).id;

    // Update user information
    let response = update(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test 1",
            "firstname": "Tutu",
            "rate_limit": 10,
        })
        .to_string(),
        &token,
        &user_id,
    )
    .await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::OK);

    let user = TestUser::from_body(&response.body.to_string());

    assert_eq!(user.lastname, String::from("Test 1"));
    assert_eq!(user.firstname, String::from("Tutu"));
    assert_eq!(user.username, String::from("test-user-creation@test.com"));
}

#[tokio::test]
async fn test_api_user_forgotten_password() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    // Create a user
    let _response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test",
            "firstname": "Toto",
            "rate_limit": 10,
        })
        .to_string(),
        &token,
    )
    .await;

    let response = forgotten_password(&app, "test-user-creation@test.com").await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::OK);

    let body = TestPasswordReset::from_body(&response.body.to_string());
    let uuid = Uuid::parse_str(&body.token).expect("invalid uuid");
    assert_eq!(uuid.get_version_num(), 4);

    let now = chrono::Utc::now();
    assert!(body.expired_at > now);
}

#[tokio::test]
async fn test_api_user_forgotten_password_email_not_found() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    // Create a user
    let _response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test",
            "firstname": "Toto",
            "rate_limit": 10,
        })
        .to_string(),
        &token,
    )
    .await;

    let response = forgotten_password(&app, "test-user-creation_1@test.com").await;
    app.drop_database().await;

    assert_eq!(response.status_code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_user_update_password() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    // Create a user
    let _response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test",
            "firstname": "Toto",
            "rate_limit": 10,
        })
        .to_string(),
        &token,
    )
    .await;

    // Get a reset password token
    let response = forgotten_password(&app, "test-user-creation@test.com").await;
    let token = TestPasswordReset::from_body(&response.body.to_string()).token;

    let response = update_password(
        &app,
        &token,
        serde_json::json!({
            "password": "11111111",
        })
        .to_string(),
    )
    .await;

    assert_eq!(response.status_code, StatusCode::OK);

    // Try to login with new password
    let response = login_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "11111111"
        })
        .to_string(),
    )
    .await;

    assert_eq!(response.status_code, StatusCode::OK);

    // Is token still in database?
    let still_in_db = is_password_reset_token_still_in_database(app.database(), &token).await;
    app.drop_database().await;

    assert!(!still_in_db);
}

#[tokio::test]
async fn test_api_user_update_password_with_old_password() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.with_state().build();
    let (_response, token) = create_and_authenticate(&app).await;

    // Create a user
    let _response = create_user_request(
        &app,
        serde_json::json!({
            "username": "test-user-creation@test.com",
            "password": "00000000",
            "lastname": "Test",
            "firstname": "Toto",
            "rate_limit": 10,
        })
        .to_string(),
        &token,
    )
    .await;

    // Get a reset password token
    let response = forgotten_password(&app, "test-user-creation@test.com").await;
    let token = TestPasswordReset::from_body(&response.body.to_string()).token;

    let response = update_password(
        &app,
        &token,
        serde_json::json!({
            "password": "00000000",
        })
        .to_string(),
    )
    .await;

    assert_eq!(response.status_code, StatusCode::BAD_REQUEST);

    // Is token still in database?
    let still_in_db = is_password_reset_token_still_in_database(app.database(), &token).await;
    app.drop_database().await;

    assert!(still_in_db);
}
