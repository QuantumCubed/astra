use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TextMessagePayload {
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct TextChunkPayload {
    pub content: String,
    pub done: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ToolCallPayload {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct ToolResultPayload {
    pub name: String,
    pub result: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorPayload {
    pub message: String,
    pub code: String,
}
#[derive(Serialize, Deserialize)]pub struct TranscriptPayload {
    pub text: String
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Message {
    TextMessage(TextMessagePayload),
    AudioEnd,
    Transcript(TranscriptPayload),
    TtsEnd,
    TextChunk(TextChunkPayload),
    ToolCall(ToolCallPayload),
    ToolResult(ToolResultPayload),
    Error(ErrorPayload),
}

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    pub request_id: Option<String>,
    #[serde(flatten)]
    pub message: Message,
}