use crate::{
    config::Config,
    databases, handlers,
    layers::{self, MakeRequestUuid, SharedState, State},
    logger, routes,
};
use axum::{error_handling::HandleErrorLayer, routing::get_service, Extension, Router};
use color_eyre::Result;
#[cfg(feature = "ws")]
use std::collections::HashSet;
#[cfg(feature = "ws")]
use std::sync::{Arc, Mutex};
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
#[cfg(feature = "ws")]
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, ServiceBuilderExt};

/// State for WebSocket chat example
#[cfg(feature = "ws")]
pub struct ChatAppState {
    pub user_set: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<String>,
}

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
    let cors = layers::cors(&settings);

    // Layers
    // ------
    let layers = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(layers::logger::LoggerLayer)
        .layer(HandleErrorLayer::new(handlers::timeout_error))
        .timeout(Duration::from_secs(settings.request_timeout))
        .propagate_x_request_id()
        .into_inner();

    // Chat app state
    // --------------
    #[cfg(feature = "ws")]
    let user_set = Mutex::new(HashSet::new());
    #[cfg(feature = "ws")]
    let (tx, _rx) = broadcast::channel(100);
    #[cfg(feature = "ws")]
    let chat_app_state = Arc::new(ChatAppState { user_set, tx });

    // Routing
    // -------
    let app = Router::new()
        .fallback(
            get_service(ServeDir::new("assets").append_index_html_on_directories(true))
                .handle_error(handlers::static_file_error),
        )
        .nest("/api/v1", routes::api().layer(cors));

    #[cfg(feature = "ws")]
    let app = app.nest("/ws", routes::ws());

    let app = app
        .nest("/", routes::web())
        .layer(layers::rate_limiter::RateLimiterLayer::new(
            &redis_pool,
            settings.jwt_secret_key.clone(),
            settings.redis_prefix.clone(),
            settings.limiter_enabled,
            settings.limiter_requests_by_second,
            settings.limiter_expire_in_seconds,
            settings.limiter_white_list.clone(),
        ));

    #[cfg(feature = "ws")]
    let app = app.layer(Extension(chat_app_state));

    let app = app
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
