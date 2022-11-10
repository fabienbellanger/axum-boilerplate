//! Prometheus metrics layer

use crate::{
    app_error,
    errors::{AppError, AppErrorCode},
    APP_NAME,
};
use axum::{extract::MatchedPath, middleware::Next, response::IntoResponse};
use hyper::Request;
use metrics::{histogram, increment_counter};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::time::Instant;

pub const SECONDS_DURATION_BUCKETS: &[f64; 11] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

pub struct PrometheusMetric {}

impl PrometheusMetric {
    /// Return a new `PrometheusHandle`
    pub fn get_handle() -> Result<PrometheusHandle, AppError> {
        PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full("http_requests_duration_seconds".to_string()),
                SECONDS_DURATION_BUCKETS,
            )
            .map_err(|err| app_error!(AppErrorCode::InternalError, err.to_string()))?
            .install_recorder()
            .map_err(|err| app_error!(AppErrorCode::InternalError, err.to_string()))
    }

    /// Layer tracking requests
    pub async fn get_layer<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
        let start = Instant::now();
        let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
            matched_path.as_str().to_owned()
        } else {
            req.uri().path().to_owned()
        };
        let method = req.method().clone();

        let response = next.run(req).await;

        let latency = start.elapsed().as_secs_f64();
        let status = response.status().as_u16().to_string();
        let labels = [
            ("method", method.to_string()),
            ("path", path),
            ("service", APP_NAME.to_owned()),
            ("status", status),
        ];

        increment_counter!("http_requests_total", &labels);
        histogram!("http_requests_duration_seconds", latency, &labels);

        response
    }
}
