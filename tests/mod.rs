mod test_helper;

use crate::test_helper::{TestAppBuilder, TestDatabase};
use axum::http::StatusCode;
use axum::{body::Body, http::Request};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check_ext() {
    let app = TestAppBuilder::new().add_web_routes().build().router;
    dbg!(TestDatabase::new());

    let response = app
        .oneshot(Request::builder().uri("/health-check").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
