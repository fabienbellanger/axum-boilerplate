//! WebSocklet handlers

use crate::server::ChatAppState;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

/// Simple WebSocket handler
pub async fn simple_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_simple_socket)
}

/// Simple WebSocket logic
async fn handle_simple_socket(mut socket: WebSocket) {
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

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

/// Chat WebSocket handler
pub async fn chat_ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<ChatAppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<ChatAppState>) {
    // By splitting we can send and receive at the same time
    let (mut sender, mut receiver) = stream.split();

    // Username gets set in the receive loop, if it's valid
    let mut username = String::new();

    // Loop until a valid username is entered
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            // If username that is sent by client is not taken, fill username string
            check_username(&state, &mut username, &name);

            // If not empty we want to quit the loop else we want to quit function
            if !username.is_empty() {
                break;
            } else {
                // Only send our client that username is taken
                let _ = sender
                    .send(Message::Text(String::from("Username already taken.")))
                    .await;

                return;
            }
        }
    }

    // Subscribe before sending joined message
    let mut rx = state.tx.subscribe();

    // Send joined message to all subscribers
    let msg = format!("{} joined.", username);
    tracing::debug!("{}", msg);
    let _ = state.tx.send(msg);

    // This task will receive broadcast messages and send text message to our client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass to the receiving task
    let tx = state.tx.clone();
    let name = username.clone();

    // This task will receive messages from client and send them to broadcast subscribers
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message
            let _ = tx.send(format!("{}: {}", name, text));
        }
    });

    // If any one of the tasks exit, abort the other
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send user left message
    let msg = format!("{} left.", username);
    tracing::debug!("{}", msg);
    let _ = state.tx.send(msg);

    // Remove username from map so new clients can take it
    state.user_set.lock().unwrap().remove(&username);
}

/// Check if username is valid and not already
fn check_username(state: &ChatAppState, string: &mut String, name: &str) {
    let mut user_set = state.user_set.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        string.push_str(name);
    }
}
