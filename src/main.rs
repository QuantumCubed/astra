use axum::{Router, routing::get};

mod backend;
mod handlers;
mod tools;

use crate::handlers::ws::ws_handler;
use crate::backend::state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
