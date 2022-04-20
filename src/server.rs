use crate::{
    config::Config,
    database,
    layers::{self, header_value_to_str, MakeRequestUuid, SharedState, State},
    logger, routes,
};
use axum::{
    http::{Request, Response},
    Extension, Router,
};
use color_eyre::Result;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer, ServiceBuilderExt};
use tracing::Span;

/// Starts API server
pub async fn start_server() -> Result<()> {
    // Install Color Eyre
    // ------------------
    color_eyre::install()?;

    // Load configuration
    // ------------------
    let settings = Config::from_env()?;

    // Tracing
    // -------
    let subscriber = logger::get_subscriber("info".to_owned(), std::io::stdout);
    logger::init_subscriber(subscriber);

    // Database
    // --------
    let pool = database::init(&settings).await?;

    // CORS
    // ----
    let cors = layers::cors();

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

    // Build our application with a single route
    let app = Router::new()
        .nest("/api/v1", routes::api().layer(cors))
        .nest("/", routes::web())
        .layer(Extension(pool))
        .layer(logger_layer)
        .layer(Extension(SharedState::new(State::init(&settings))));

    // Start server
    let addr = format!("{}:{}", settings.server_url, settings.server_port);
    info!("Starting server on {}", &addr);
    Ok(axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?)
}
