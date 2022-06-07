//! WebSocklet handlers

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

/// WebSocket handler
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// WebSocket logic
async fn handle_socket(mut socket: WebSocket) {
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    println!("client sent str: {:?}", t);
                }
                Message::Binary(_) => {
                    println!("client sent binary data");
                }
                Message::Ping(_) => {
                    println!("socket ping");
                }
                Message::Pong(_) => {
                    println!("socket pong");
                }
                Message::Close(_) => {
                    println!("client disconnected");
                    return;
                }
            }
        } else {
            println!("client disconnected");
            return;
        }
    }

    let mut i = 0;
    loop {
        i += 1;

        if socket.send(Message::Text(format!("Hi! - {}", i))).await.is_err() {
            println!("client disconnected");
            return;
        }

        if i == 10 {
            socket.close().await.expect("error during connection close");
            return;
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
