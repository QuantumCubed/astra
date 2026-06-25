use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::tools::registry::Tool;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OllamaMessage {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub stream: bool,
    pub messages: Vec<OllamaMessage>,
    // Shared from AppState (read-only) so building a request per loop iteration is a cheap
    // Arc clone, not a deep copy of every tool schema. Serializes identically to `Vec<Tool>`.
    pub tools: Arc<Vec<Tool>>,
}
