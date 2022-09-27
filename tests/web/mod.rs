use super::helper::TestAppBuilder;
use axum::http::StatusCode;
use axum::{body::Body, http::Request};
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