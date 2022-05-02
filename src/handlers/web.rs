//! Web handlers

use std::time::Duration;
use tokio::time::sleep;

// Route: GET "/health-check"
pub async fn health_check<'a>() -> &'a str {
    "OK"
}

// Route: GET "/timeout"
pub async fn timeout() {
    sleep(Duration::from_secs(30)).await;
}

// Route: GET "/spawn"
pub async fn spawn() {
    info!("Spawn start");
    tokio::spawn(async move {
        sleep(Duration::from_secs(2)).await;
        info!("Spawn end");
    });
    info!("Spawn return");
}

// Route: GET "/stream"
pub async fn stream() {}
