use serde_json::Value;

use crate::handlers::req_handler::{ChatRequest, GenRequest};

const BASE_URL: &str = "http://10.244.35.194:11434";

pub async fn list_models() -> Result<Value, reqwest::Error> {
    let response = reqwest::get(format!("{BASE_URL}/api/tags"))
        .await?
        .json::<Value>()
        .await?;

    Ok(response)
}

pub async fn generate(req: GenRequest) -> Result<Value, reqwest::Error> {
    let response = reqwest::Client::new()
        .post(format!("{BASE_URL}/api/generate"))
        .json(&req)
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(response)
}

pub async fn chat(req: ChatRequest) -> Result<reqwest::Response, reqwest::Error> {
    let response = reqwest::Client::new()
        .post(format!("{BASE_URL}/api/chat"))
        .json(&req)
        .send()
        .await?;

    Ok(response)
}