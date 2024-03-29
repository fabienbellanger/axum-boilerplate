use crate::{
    config::{databases, logger, Config},
    handlers,
    layers::{
        self, basic_auth::BasicAuthLayer, prometheus::PrometheusMetric, rate_limiter::RateLimiterLayer, ChatState,
        MakeRequestUuid, SharedChatState, SharedState, State,
    },
    routes,
};
use axum::{error_handling::HandleErrorLayer, middleware, routing::get, Extension, Router};
use color_eyre::Result;
use std::collections::HashSet;
use std::time::Duration;
use std::{future::ready, sync::Mutex};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, ServiceBuilderExt};

/// Starts API server
pub async fn start_server() -> Result<()> {
    // Install Color Eyre
    // ------------------
    color_eyre::install()?;

    // Load configuration
    // ------------------
    let settings = Config::from_env()?;

    let app = get_app(&settings).await?;

    // Start server
    // ------------
    let addr = format!("{}:{}", settings.server_url, settings.server_port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Starting server on {}...", &addr);

    let server = axum::serve(listener, app);

    // No graceful shutdown in development environment
    if &settings.environment == "development" {
        Ok(server.await?)
    } else {
        Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
    }
}

pub async fn get_app(settings: &Config) -> Result<Router> {
    // Tracing
    // -------
    logger::init(&settings.environment, &settings.logs_path, &settings.logs_file)?;

    // MySQL database
    // --------------
    let pool = databases::init_mysql(settings).await?;

    // CORS
    // ----
    let cors = layers::cors(settings);

    // Layers
    // ------
    let layers = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(layers::logger::LoggerLayer)
        .layer(HandleErrorLayer::new(handlers::timeout_error))
        .timeout(Duration::from_secs(settings.request_timeout))
        .propagate_x_request_id();

    // Global state
    // ------------
    let global_state = SharedState::new(State::init(settings));

    // Routing - API
    // -------------
    let mut app = Router::new().nest("/api/v1", routes::api(global_state.clone()).layer(cors));

    // Routing - WS
    // ------------
    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);
    let chat_state = SharedChatState::new(ChatState { user_set, tx });
    app = app.nest("/ws", routes::ws(chat_state));

    // Routing - Web
    // -------------
    app = app.nest("/", routes::web(settings));

    // Prometheus metrics
    // ------------------
    if settings.prometheus_metrics_enabled {
        let handle = PrometheusMetric::get_handle()?;
        app = app
            .nest(
                "/metrics",
                Router::new().route(
                    "/",
                    get(move || ready(handle.render())).layer(BasicAuthLayer::new(
                        &settings.basic_auth_username,
                        &settings.basic_auth_password,
                    )),
                ),
            )
            .route_layer(middleware::from_fn(PrometheusMetric::get_layer));
    }

    // Rate limiter
    // ------------
    if settings.limiter_enabled {
        // Redis
        // -----
        let redis_pool = databases::init_redis(settings).await?;

        app = app
            .layer(RateLimiterLayer::new(
                global_state.clone(),
                redis_pool.clone(),
                settings.redis_prefix.clone(),
                settings.limiter_requests_by_second,
                settings.limiter_expire_in_seconds,
                settings.limiter_white_list.clone(),
            ))
            .layer(Extension(redis_pool));
    }

    app = app
        .fallback_service(ServeDir::new("assets").append_index_html_on_directories(true)) // FIXME: static_file_error not work this Axum 0.6.9!
        .layer(middleware::from_fn(layers::override_http_errors))
        .layer(Extension(pool))
        .layer(layers);

    let app = app.with_state(global_state);

    Ok(app)
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
