use crate::api::helpers::user::login_request;
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
