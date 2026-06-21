use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TextMessagePayload {
    content: String
}

#[derive(Serialize, Deserialize)]
pub struct TextChunkPayload {
    content: String,
    done: bool
}

#[derive(Serialize, Deserialize)]
pub struct ToolCallPayload {
    name: String,
    args: serde_json::Value
}

#[derive(Serialize, Deserialize)]
pub struct ToolResultPayload {
    name: String,
    result: String
}

#[derive(Serialize, Deserialize)]
pub struct ErrorPayload {
    message: String,
    code: String
}

// Audio: handled as binary WebSocket frames, not JSON messages
// Binary frames will be defined when Phase 3 is implemented

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Message {
    TextMessage(TextMessagePayload),
    TextChunk(TextChunkPayload),
    ToolCall(ToolCallPayload),
    ToolResult(ToolResultPayload),
    Error(ErrorPayload)
}

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    request_id: Option<String>,
    #[serde(flatten)]
    message: Message,
}