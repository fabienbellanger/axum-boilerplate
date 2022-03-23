use std::time::Duration;

use axum::{
    http::{HeaderValue, Method, Request, Response},
    routing::get,
    Router,
};
use axum_boilerplate::{config::Config, logger};
use color_eyre::Result;
use tower::ServiceBuilder;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer, ServiceBuilderExt};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestId, RequestId},
};
use tracing::Span;
use uuid::Uuid;

#[macro_use]
extern crate tracing;

// Request ID middleware
// TODO: Put in middlewares module
#[derive(Clone, Copy)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string().parse().unwrap();
        Some(RequestId::new(request_id))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install Color Eyre
    // ------------------
    color_eyre::install()?;

    // Load configuration
    // ------------------
    let settings = Config::from_env()?;
    // tracing_subscriber::fmt::init();
    let subscriber = logger::get_subscriber("info".to_owned(), std::io::stdout);
    logger::init_subscriber(subscriber);

    // Logger
    // ------
    // TODO: Continue
    let logger_layer = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
                // .make_span_with(|_request: &Request<_>| {
                //     tracing::info_span!("HTTP", status_code = tracing::field::Empty)
                // })
                .on_request(|request: &Request<_>, _span: &Span| {
                    // TODO: Remove Option
                    info!(
                        r#"[REQUEST] method: {}, host: {:?}, uri: {}, request_id: {:?}, user_agent: {:?}"#,
                        request.method(),
                        request.headers().get("host").unwrap_or(&HeaderValue::from_static("")),
                        request.uri(),
                        request
                            .headers()
                            .get("x-request-id")
                            .unwrap_or(&HeaderValue::from_static("")),
                        request
                            .headers()
                            .get("user-agent")
                            .unwrap_or(&HeaderValue::from_static(""))
                    );
                })
                .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                    // _span.record("status_code", &tracing::field::display(_response.status()));
                    info!(
                        "[RESPONSE] status_code: {}, request_id: {:?}, latency: {:?}",
                        response.status().as_u16(),
                        response
                            .headers()
                            .get("x-request-id")
                            .unwrap_or(&HeaderValue::from_static("")),
                        latency
                    );
                })
                .on_failure(|error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                    error!("[FAILURE] failure: {:?}", error);
                }),
        )
        .propagate_x_request_id()
        .into_inner();

    // CORS
    // ----
    let cors = CorsLayer::new()
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_origin(Any);

    // Build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(cors)
        .layer(logger_layer);

    // Run it with hyper
    let addr = format!("{}:{}", settings.server_url, settings.server_port);
    info!("Starting server on {}", &addr);
    Ok(axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?)
}
