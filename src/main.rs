use axum::{Router, routing::get};

mod backend;
mod handlers;
mod tools;
mod integrations;

use crate::handlers::ws::ws_handler;
use crate::backend::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("loading models (first run downloads weights — this can take a while)…");
    let state = AppState::new().await;

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    tracing::info!("astra listening on 0.0.0.0:3000");
    axum::serve(listener, app)
        .await
        .expect("server error");
}
