use axum::{
    http::{HeaderValue, Method, Request, Response},
    Extension, Router,
};
use axum_boilerplate::{config::Config, database, logger, routes};
use color_eyre::Result;
use std::str::from_utf8;
use std::time::Duration;
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

/// Convert `HeaderValue` to `&str`
fn header_value_to_str(value: Option<&HeaderValue>) -> &str {
    match value {
        Some(value) => from_utf8(value.as_bytes()).unwrap_or(""),
        None => "",
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
    let subscriber = logger::get_subscriber("info".to_owned(), std::io::stdout);
    logger::init_subscriber(subscriber);

    // Database
    // --------
    let pool = database::init(&settings).await?;

    // Logger
    // ------
    let logger_layer = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<_>, _span: &Span| {
                    info!(
                        r#"[REQUEST] method: {}, host: {}, uri: {}, request_id: {}, user_agent: {}"#,
                        request.method(),
                        header_value_to_str(request.headers().get("host")),
                        request.uri(),
                        header_value_to_str(request.headers().get("x-request-id")),
                        header_value_to_str(request.headers().get("user-agent"))
                    );
                })
                .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                    // _span.record("status_code", &tracing::field::display(_response.status()));
                    info!(
                        "[RESPONSE] status_code: {}, request_id: {}, latency: {:?}",
                        response.status().as_u16(),
                        header_value_to_str(response.headers().get("x-request-id")),
                        latency,
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
        .nest("/api/v1", routes::api())
        .nest("/", routes::web())
        .layer(Extension(pool))
        .layer(cors)
        .layer(logger_layer);

    // Start server
    let addr = format!("{}:{}", settings.server_url, settings.server_port);
    info!("Starting server on {}", &addr);
    Ok(axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?)
}
