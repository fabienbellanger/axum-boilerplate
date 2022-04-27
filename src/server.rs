use crate::{
    config::Config,
    database,
    layers::{self, MakeRequestUuid, SharedState, State},
    logger, routes,
};
use axum::{Extension, Router};
use color_eyre::Result;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

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
    logger::init(&settings.environment, &settings.logs_path, &settings.logs_file);

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
        .layer(crate::layers::logger::LoggerLayer)
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
