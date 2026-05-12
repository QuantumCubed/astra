use serde_json::Value;
use serde::Serialize;

pub async fn list_models() -> Result<Value, reqwest::Error> {
    let response = reqwest::get("http://10.244.35.194:11434/api/tags")
    .await?
    .json::<Value>()
    .await?;
    
    Ok(response)
}

#[derive(Serialize, Debug)]
struct GenRequest {
    model: String,
    stream: bool,
    prompt: String 
}

pub async fn generate(prompt: &str) -> Result<Value, reqwest::Error> {

    let req = GenRequest {
        model: "qwen3.5:9b".to_string(),
        stream: false,
        prompt: prompt.to_string()
    };

    let client = reqwest::Client::new();
    let response = client.post("http://10.244.35.194:11434/api/generate")
        .json(&req)
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(response)
}
