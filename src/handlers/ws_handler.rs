use axum::{extract::{State, WebSocketUpgrade, ws::WebSocket, ws::Message as WsFrame}, response::Response};
use crate::{conversation::Conversation, handlers::req_handler::ChatRequest, ollama_client::async_client, protocol::{Envelope, Message, TextChunkPayload}};

use crate::state::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket(mut socket: WebSocket, state: AppState) {

    let mut conversation = Conversation::new(state.system_prompt.clone()); // hardcoded max tokens for now

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            WsFrame::Text(text) => {
                match serde_json::from_str::<Envelope>(&text) {
                    Ok(envelope) => {
                        // match on envelop.message
                        let request_id = envelope.request_id.clone();
                        match envelope.message {
                            Message::TextMessage(payload) => {
                                conversation.add_user_turn(&payload.content);

                                let req = ChatRequest {
                                    model: "qwen3.5:9b".to_string(),
                                    stream: false,
                                    messages: conversation.messages().to_vec(),
                                    tools: state.tools.clone()
                                };

                                if let Ok(res) = async_client::chat(req).await {
                                    let content = res["message"]["content"]
                                    .as_str()
                                    .unwrap_or_default()
                                    .to_string();

                                    conversation.add_astra_turn(&content);

                                    let reply = Envelope {
                                    request_id,
                                    message: Message::TextChunk(TextChunkPayload { content, done: true})
                                    };

                                    if let Ok(json) = serde_json::to_string(&reply) {
                                        let _ = socket.send(WsFrame::Text(json.into())).await;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {}
                }
            }

            WsFrame::Close(_) => return,
            _ => {} // ignore ping/pong binary frames
        }
    }
}