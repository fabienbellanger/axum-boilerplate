mod helper;

use crate::helper::{TestAppBuilder, TestDatabase};
use axum::http::StatusCode;
use axum::{body::Body, http::Request};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = TestAppBuilder::new().add_web_routes().build().router;
    let test_db = TestDatabase::new().await;
    dbg!(&test_db.database().await);
    test_db.drop_database().await;

    let response = app
        .oneshot(Request::builder().uri("/health-check").body(Body::empty()).unwrap())
        .await
        .unwrap();

    // assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
