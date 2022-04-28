use crate::{
    config::Config,
    database,
    layers::{self, MakeRequestUuid, SharedState, State},
    logger, routes,
};
use axum::{Extension, Router};
use color_eyre::Result;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

// TODO: Timeout: https://docs.rs/axum/latest/axum/error_handling/index.html#applying-fallible-middleware

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
        //.timeout(Duration::from_secs(10)) // Does not work
        .layer(crate::layers::logger::LoggerLayer)
        .propagate_x_request_id()
        .into_inner();

    // Routing
    // -------
    let app = Router::new()
        .nest("/api/v1", routes::api().layer(cors))
        .nest("/", routes::web())
        .layer(Extension(pool))
        .layer(logger_layer)
        .layer(Extension(SharedState::new(State::init(&settings))));

    // Start server
    // ------------
    let addr = format!("{}:{}", settings.server_url, settings.server_port);
    info!("Starting server on {}", &addr);
    Ok(axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}
