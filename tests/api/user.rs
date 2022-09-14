use crate::helper::{TestApp, TestAppBuilder};
use axum::http::Request;
use axum::http::StatusCode;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn test_api_login_unauthorized_user() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.build();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(
                    serde_json::json!({
                        "username": "test@gmail.com",
                        "password": "00000000"
                    })
                    .to_string()
                    .into(),
                )
                .unwrap(),
        )
        .await
        .unwrap();

    app.drop_database().await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        body,
        serde_json::json!({
            "code": 401,
            "message": "Unauthorized"
        })
    );
}
