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

    // TODO: Add .env file and custom logger formatter like actix-web
    // Load configuration
    // ------------------
    let _settings = Config::from_env()?;
    tracing_subscriber::fmt::init();

    let middleware_stack = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .on_request(|_request: &Request<_>, _span: &Span| {
                    info!("request={:?}", _request);
                })
                .on_response(|_response: &Response<_>, _latency: Duration, _span: &Span| {
                    info!("response={:?}, latency={:?}", _response, _latency);
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

    // Run it with hyper on localhost:3000
    // TODO: Use config var
    info!("Starting server on 0.0.0.0:3000");
    Ok(axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?)
}
