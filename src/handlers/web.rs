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

/// Simulate a long process
async fn long_process() {
    sleep(Duration::from_secs(2)).await;
    info!("long process end");
}

// Route: GET "/spawn"
pub async fn spawn() {
    info!("Spawn start");
    tokio::spawn(long_process());
    info!("Spawn return");
}

// Route: GET "/stream"
pub async fn stream() {}
