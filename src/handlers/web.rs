//! Web handlers

use crate::errors::AppError;
use askama::Template;
use axum::{
    body::StreamBody,
    extract::Path,
    http::header::CONTENT_TYPE,
    response::{AppendHeaders, IntoResponse},
    Json,
};
use bytes::{Bytes, BytesMut};
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;

// Route: GET "/health-check"
pub async fn health_check<'a>() -> &'a str {
    "OK"
}

#[derive(Template)]
#[template(path = "hello.html")]
pub struct HelloTemplate<'a> {
    name: &'a str,
}

// Route: GET "/hello/:name"
// TODO: Waiting for askama 0.11.2 to fix bug
pub async fn hello(Path(name): Path<&'static str>) -> HelloTemplate<'static> {
    HelloTemplate { name }
}

// Route: GET "/timeout"
pub async fn timeout() {
    sleep(Duration::from_secs(20)).await;
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

#[derive(Debug, Serialize)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Task {
    fn new(i: usize) -> Self {
        Self {
            id: i,
            name: format!("My task with number: {}", i),
            created_at: chrono::Utc::now(),
        }
    }
}

// Route: GET "/big-json"
pub async fn big_json() -> Json<Vec<Task>> {
    let mut tasks = Vec::new();

    for i in 0..100_000 {
        tasks.push(Task::new(i + 1));
    }

    Json(tasks)
}

// Route: GET "/stream"
pub async fn stream() -> impl IntoResponse {
    let stream_tasks = async_stream::stream! {
        let mut bytes = BytesMut::new();

        bytes.extend_from_slice("[".as_bytes());
        let byte = bytes.split().freeze();
        yield Ok::<Bytes, AppError>(byte);

        // From sqlx result:
        // let mut i = 0;
        // while let Some(row) = tasks.try_next().await? {
        //     if i > 0 {
        //         bytes.extend_from_slice(",".as_bytes());
        //     }
        //     i += 1;
        //     match row {
        //         Ok(row) => match serde_json::to_string(&row) {
        //                 Ok(task) => {
        //                     bytes.extend_from_slice(task.as_bytes());
        //                     let byte = bytes.split().freeze();
        //                     yield Ok::<Bytes, AppError>(byte)
        //                 },
        //                 Err(err) => error!("Tasks list stream error: {}", err)
        //             },
        //         Err(err) => error!("Tasks list stream error: {}", err)
        //     }
        // }

        for i in 0..100_000 {
            if i > 0 {
                bytes.extend_from_slice(",".as_bytes());
            }

            let task = Task::new(i + 1);

            match serde_json::to_string(&task) {
                Ok(task) => {
                    bytes.extend_from_slice(task.as_bytes());
                    let byte = bytes.split().freeze();
                    yield Ok::<Bytes, AppError>(byte)
                },
                Err(err) => error!("Tasks list stream error: {}", err)
            }
        }

        bytes.extend_from_slice("]".as_bytes());
        let byte = bytes.split().freeze();
        yield Ok::<Bytes, AppError>(byte);
    };

    (
        AppendHeaders([(CONTENT_TYPE, "application/json")]),
        StreamBody::new(stream_tasks),
    )
}
