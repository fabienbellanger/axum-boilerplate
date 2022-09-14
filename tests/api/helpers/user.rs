//! Helpers for user API tests

use super::TestResponse;
use crate::helper::TestApp;
use axum::http::Request;
use serde_json::Value;
use std::collections::HashMap;
use tower::ServiceExt;

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
