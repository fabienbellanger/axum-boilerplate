mod helper;

use crate::helper::TestAppBuilder;
use axum::http::StatusCode;
use axum::{body::Body, http::Request};
use helper::TestApp;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = TestAppBuilder::new().add_web_routes().build().router;
    let response = app
        .oneshot(Request::builder().uri("/health-check").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_login() {
    let app: TestApp = TestAppBuilder::new().add_api_routes().await.build();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/login")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    app.drop_database().await;

    assert_eq!(response.status(), StatusCode::OK);
}
