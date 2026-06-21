use axum::{extract::{State, WebSocketUpgrade, ws::WebSocket, ws::Message as WsFrame}, response::Response};
use crate::{conversation::Conversation, handlers::req_handler::ChatRequest, ollama_client::async_client, protocol::{Envelope, Message, TextChunkPayload}};

use crate::state::AppState;
use futures::StreamExt;

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
                                    stream: true,
                                    messages: conversation.messages().to_vec(),
                                    tools: state.tools.clone()
                                };

                                if let Ok(res) = async_client::chat(req).await {
                                    let mut stream = res.bytes_stream();
                                    let mut line_buf = String::new();
                                    let mut full_content = String::new();

                                    while let Some(Ok(chunk)) = stream.next().await {
                                        line_buf.push_str(&String::from_utf8_lossy(&chunk));
                                        while let Some(pos) = line_buf.find('\n') {
                                            let line: String = line_buf.drain(..=pos).collect();
                                            let line = line.trim();
                                            
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                                                let token = json["message"]["content"].as_str().unwrap_or("");
                                                let done = json["done"].as_bool().unwrap_or(false);
                                                
                                                full_content.push_str(token);

                                                if !token.is_empty() || done {
                                                    let reply = Envelope {
                                                        request_id: request_id.clone(),
                                                        message: Message::TextChunk(TextChunkPayload { content: token.to_string(), done })
                                                    };
                                                    
                                                    if let Ok(json_str) = serde_json::to_string(&reply) {
                                                        let _ = socket.send(WsFrame::Text(json_str.into())).await;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    conversation.add_astra_turn(&full_content);
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