pub mod user;

use crate::helper::TestApp;
use axum::body::Body;
use axum::http::StatusCode;
use axum_boilerplate::utils::errors::AppErrorMessage;
use http_body_util::BodyExt;
use hyper::Request;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tower::ServiceExt;

/// HTTP response for test
#[derive(Debug)]
pub struct TestResponse {
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

impl TestResponse {
    /// Create a new `TestResponse`
    pub async fn new(app: &TestApp, url: &str, method: &str, body: Option<String>, token: Option<&str>) -> Self {
        let mut request = Request::builder()
            .uri(url)
            .method(method)
            .header("Content-Type", "application/json");
        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let request = request.body(match body {
            None => Body::empty(),
            Some(body) => body.into(),
        });

        let response = app.router.clone().oneshot(request.unwrap()).await.unwrap();

        let status_code = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("failed to convert body into bytes")
            .to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

        TestResponse {
            status_code,
            body,
            headers: HashMap::new(),
        }
    }
}

impl TryInto<AppErrorMessage> for TestResponse {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<AppErrorMessage, Self::Error> {
        serde_json::from_str(&self.body.to_string())
    }
}

#[derive(Deserialize)]
pub struct TestPaginateResponse<T> {
    pub data: T,
    pub total: i64,
}
