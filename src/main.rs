use axum::Json;
use axum::response::IntoResponse;
use axum::{
    routing::get, 
    routing::post,
    Router,
};

use serde::Deserialize;

mod handlers;
mod ollama_client;
mod tools;
mod state;

use crate::state::AppState;
use handlers::req_handler::call_model;
use handlers::req_handler::generate_model;
use handlers::req_handler::chat_model;
use tokio::sync::watch;

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

pub async fn chat_handler(Json(payload): Json<ChatRequest>) -> impl IntoResponse {
    println!("{}", payload.message);
}

#[tokio::main]
async fn main() {

    let state = AppState::new();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ollama", get(call_model))
        .route("/generate", post(generate_model))
        .route("/chat", post(chat_model))
        .with_state(state);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
