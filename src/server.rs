use crate::{
    config::Config,
    databases, handlers,
    layers::{self, MakeRequestUuid, SharedState, State},
    logger, routes,
};
use axum::{error_handling::HandleErrorLayer, Extension, Router};
use color_eyre::Result;
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
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
    logger::init(&settings.environment, &settings.logs_path, &settings.logs_file)?;

    // Database
    // --------
    let pool = databases::init(&settings).await?;

    // Redis
    // -----
    let redis_pool = databases::init_redis(&settings).await?;

    // CORS
    // ----
    let cors = layers::cors();

    // Layers
    // ------
    let layers = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(layers::logger::LoggerLayer)
        .layer(HandleErrorLayer::new(handlers::timeout_error))
        .timeout(Duration::from_secs(settings.request_timeout))
        .propagate_x_request_id()
        .into_inner();

    // Routing
    // -------
    let app = Router::new()
        .nest("/api/v1", routes::api().layer(cors))
        .nest("/", routes::web())
        .layer(layers::rate_limiter::RateLimiterLayer {
            pool: &redis_pool,
            jwt_secret: settings.jwt_secret_key.clone(),
        })
        .layer(Extension(pool))
        .layer(Extension(redis_pool))
        .layer(layers)
        .layer(Extension(SharedState::new(State::init(&settings))));

    // Start server
    // ------------
    let addr = format!("{}:{}", settings.server_url, settings.server_port);
    info!("Starting server on {}", &addr);
    Ok(axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
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
