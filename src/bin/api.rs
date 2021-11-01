use axum::{
    handler::get,
    http::{Request, Response},
    Router,
};
use axum_boilerplate::config::Config;
use color_eyre::Result;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> Result<()> {
    // Install Color Eyre
    // ------------------
    color_eyre::install()?;

    // TODO: Add custom logger formatter like actix-web
    // Load configuration
    // ------------------
    let settings = Config::from_env()?;
    tracing_subscriber::fmt::init();

    let middleware_stack = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|_request: &Request<_>| {
                    tracing::info_span!("http-request", status_code = tracing::field::Empty,)
                })
                .on_request(|_request: &Request<_>, _span: &Span| {
                    info!("{:?}", _request);
                })
                .on_response(|_response: &Response<_>, _latency: Duration, span: &Span| {
                    span.record("status_code", &tracing::field::display(_response.status()));
                    info!("{:?}, latency={:?}", _response, _latency);
                })
                .on_failure(|_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                    error!("failure={:?}", _error);
                }),
        )
        .into_inner();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(middleware_stack);

    // Run it with hyper
    let addr = format!("{}:{}", settings.server_url, settings.server_port);
    info!("Starting server on {}", &addr);
    Ok(axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?)
}
