use axum::extract::ws::WebSocket;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use axum::{Router, routing::get};

mod handlers;
mod ollama_client;
mod state;
mod tools;
mod protocol;
mod conversation;
mod config;

use crate::handlers::ws_handler::ws_handler;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", get(ws_handler))
        .with_state(state);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
