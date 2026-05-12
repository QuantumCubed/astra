use axum::Json;
use serde::{Deserialize};
use serde_json::Value;

use crate::ollama_client::async_client;

use axum::{http::StatusCode, response::{IntoResponse, Response}};

pub struct AppError(reqwest::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

// Now you can use ? cleanly
pub async fn call_model() -> Result<Json<Value>, AppError> {
    let data = async_client::list_models().await.map_err(AppError)?;
    Ok(Json(data))
}

pub async fn generate_model(prompt: &str) -> Result<Json<Value>, AppError> {
    let data = async_client::generate(prompt).await.map_err(AppError)?;
    Ok(Json(data))
}
