use super::types::ChatRequest;

pub async fn chat(
    client: &reqwest::Client,
    base_url: &str,
    req: ChatRequest,
) -> Result<reqwest::Response, reqwest::Error> {
    client
        .post(format!("{base_url}/api/chat"))
        .json(&req)
        .send()
        .await
}
