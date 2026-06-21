use axum::extract::State;
use axum::Json;
use axum::{http::StatusCode, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::tools::dispatch;

use crate::ollama_client::async_client;
use crate::state::AppState;
use crate::tools::registry::Tool;

// --- Error type ---

pub struct AppError(reqwest::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

// --- Request / payload types ---

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct GenRequest {
    pub model: String,
    pub stream: bool,
    pub prompt: String,
    pub tools: Vec<Tool>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub stream: bool,
    pub messages: Vec<OllamaMessage>,
    pub tools: Vec<Tool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

// --- Handlers ---

pub async fn call_model() -> Result<Json<Value>, AppError> {
    let data = async_client::list_models().await.map_err(AppError)?;
    Ok(Json(data))
}

pub async fn generate_model(
    State(state): State<AppState>,
    Json(payload): Json<GenerateRequest>,
) -> Result<Json<Value>, AppError> {
    let req = GenRequest {
        model: "qwen3.5:9b".to_string(),
        stream: false,
        prompt: payload.prompt,
        tools: state.tools.clone(),
    };

    let data = async_client::generate(req).await.map_err(AppError)?;
    Ok(Json(data))
}

// pub async fn chat_model(
//     State(state): State<AppState>,
//     Json(payload): Json<GenerateRequest>,
// ) -> Result<Json<Value>, AppError> {
//     // println!("{:#?}", state.tools);
//     let req = ChatRequest {
//         model: "qwen3.5:9b".to_string(),
//         stream: false,
//         messages: vec![OllamaMessage {
//             role: "user".to_string(),
//             content: payload.prompt,
//         }],
//         tools: state.tools.clone(),
//     };

//     let response = async_client::chat(req).await.map_err(AppError)?;
    
//     // check if the model wants to call a tool
//     if let Some(tool_calls) = response["message"]["tool_calls"].as_array() {
//         let mut results = vec![];

//         for tool_call in tool_calls {
//             let tool_name = tool_call["function"]["name"]
//                 .as_str()
//                 .unwrap_or_default();

//             let args = tool_call["function"]["arguments"].clone();

//             let result = dispatch::dispatch_tool(tool_name, args).await;
//             results.push(result);
//         }

//         return Ok(Json(serde_json::json!({ "tool_results": results })));
//     }
//     // no tool call, just return the message
//     Ok(Json(response))
// }