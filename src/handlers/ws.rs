use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket, ws::Message as WsFrame},
    response::Response,
};
use crate::{
    backend::{
        audio::{stt::transcribe, tts::synthesize},
        conversation::Conversation,
        ollama::{client, types::ChatRequest},
        protocol::{Envelope, ErrorPayload, Message, TextChunkPayload, TranscriptPayload, TtsEndPayload},
        state::AppState,
    },
    tools::dispatch,
};
use futures::StreamExt;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut conversation = Conversation::new(state.system_prompt.clone());
    let mut audio_buffer: Vec<u8> = Vec::new();

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            WsFrame::Text(text) => {
                match serde_json::from_str::<Envelope>(&text) {
                    Ok(envelope) => {
                        let request_id = envelope.request_id.clone();
                        match envelope.message {
                            Message::TextMessage(payload) => {
                                conversation.add_user_turn(&payload.content);
                                let response = run_agent_loop(&mut socket, &state, &mut conversation, request_id.clone()).await;
                                if payload.voice_response {
                                    match synthesize(state.tts_model.clone(), &state.tts_voice, response).await {
                                        Ok(samples) => {
                                            let bytes: Vec<u8> = samples
                                                .iter()
                                                .flat_map(|s| s.to_le_bytes())
                                                .collect();
                                            let _ = socket.send(WsFrame::Binary(bytes.into())).await;
                                            let tts_end = Envelope {
                                                request_id: request_id.clone(),
                                                message: Message::TtsEnd(TtsEndPayload {
                                                    sample_rate: state.tts_sample_rate,
                                                    channels: 1,
                                                    format: "f32le".to_string(),
                                                }),
                                            };
                                            if let Ok(s) = serde_json::to_string(&tts_end) {
                                                let _ = socket.send(WsFrame::Text(s.into())).await;
                                            }
                                        }
                                        Err(e) => {
                                            send_error(&mut socket, request_id, "TTS_ERROR", &format!("TTS failed: {e}")).await;
                                        }
                                    }
                                }
                            }
                            Message::AudioEnd => {
                                let audio = std::mem::take(&mut audio_buffer);
                                match transcribe(state.whisper_ctx.clone(), audio).await {
                                    Ok(transcript) => {
                                        let reply = Envelope {
                                            request_id: request_id.clone(),
                                            message: Message::Transcript(TranscriptPayload {
                                                text: transcript.clone(),
                                            }),
                                        };
                                        if let Ok(s) = serde_json::to_string(&reply) {
                                            let _ = socket.send(WsFrame::Text(s.into())).await;
                                        }

                                        conversation.add_user_turn(&transcript);
                                        let response = run_agent_loop(
                                            &mut socket,
                                            &state,
                                            &mut conversation,
                                            request_id.clone(),
                                        )
                                        .await;

                                        match synthesize(state.tts_model.clone(), &state.tts_voice, response).await {
                                            Ok(samples) => {
                                                let bytes: Vec<u8> = samples
                                                    .iter()
                                                    .flat_map(|s| s.to_le_bytes())
                                                    .collect();
                                                let _ = socket
                                                    .send(WsFrame::Binary(bytes.into()))
                                                    .await;
                                                let tts_end = Envelope {
                                                    request_id: request_id.clone(),
                                                    message: Message::TtsEnd(TtsEndPayload {
                                                    sample_rate: state.tts_sample_rate,
                                                    channels: 1,
                                                    format: "f32le".to_string(),
                                                }),
                                                };
                                                if let Ok(s) = serde_json::to_string(&tts_end) {
                                                    let _ = socket
                                                        .send(WsFrame::Text(s.into()))
                                                        .await;
                                                }
                                            }
                                            Err(e) => {
                                                send_error(
                                                    &mut socket,
                                                    request_id,
                                                    "TTS_ERROR",
                                                    &format!("TTS failed: {e}"),
                                                )
                                                .await;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        send_error(
                                            &mut socket,
                                            request_id,
                                            "STT_ERROR",
                                            &format!("transcription failed: {e}"),
                                        )
                                        .await;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        let reply = Envelope {
                            request_id: None,
                            message: Message::Error(ErrorPayload {
                                message: format!("invalid message format: {e}"),
                                code: "PARSE_ERROR".to_string(),
                            }),
                        };
                        if let Ok(json_str) = serde_json::to_string(&reply) {
                            let _ = socket.send(WsFrame::Text(json_str.into())).await;
                        }
                    }
                }
            }

            WsFrame::Binary(bytes) => {
                audio_buffer.extend_from_slice(&bytes);
            }

            WsFrame::Close(_) => return,
            _ => {}
        }
    }
}

async fn send_error(socket: &mut WebSocket, request_id: Option<String>, code: &str, message: &str) {
    let reply = Envelope {
        request_id,
        message: Message::Error(ErrorPayload {
            message: message.to_string(),
            code: code.to_string(),
        }),
    };
    if let Ok(s) = serde_json::to_string(&reply) {
        let _ = socket.send(WsFrame::Text(s.into())).await;
    }
}

async fn run_agent_loop(
    socket: &mut WebSocket,
    state: &AppState,
    conversation: &mut Conversation,
    request_id: Option<String>,
) -> String {
    loop {
        let req = ChatRequest {
            model: "qwen3.5:9b".to_string(),
            stream: true,
            messages: conversation.messages().to_vec(),
            tools: state.tools.clone(),
        };

        if let Ok(res) = client::chat(&state.client, &state.ollama_url, req).await {
            let mut stream = res.bytes_stream();
            let mut line_buf = String::new();
            let mut full_content = String::new();
            let mut tool_calls: Vec<serde_json::Value> = Vec::new();

            while let Some(Ok(chunk)) = stream.next().await {
                line_buf.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(pos) = line_buf.find('\n') {
                    let line: String = line_buf.drain(..=pos).collect();
                    let line = line.trim();

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(calls) = json["message"]["tool_calls"].as_array() {
                            tool_calls.extend(calls.iter().cloned());
                        }

                        let token = json["message"]["content"].as_str().unwrap_or("");
                        let done = json["done"].as_bool().unwrap_or(false);

                        full_content.push_str(token);

                        if !token.is_empty() || done {
                            let reply = Envelope {
                                request_id: request_id.clone(),
                                message: Message::TextChunk(TextChunkPayload {
                                    content: token.to_string(),
                                    done,
                                }),
                            };
                            if let Ok(json_str) = serde_json::to_string(&reply) {
                                let _ = socket.send(WsFrame::Text(json_str.into())).await;
                            }
                        }
                    }
                }
            }

            if !tool_calls.is_empty() {
                conversation.add_astra_tool_call(tool_calls.clone());
                for tool_call in &tool_calls {
                    let name = tool_call["function"]["name"].as_str().unwrap_or_default();
                    let args = tool_call["function"]["arguments"].clone();
                    let result = dispatch::dispatch_tool(name, args).await;
                    let result_str =
                        serde_json::to_string(&result).unwrap_or_else(|_| "error".to_string());
                    conversation.add_tool_result(&result_str);
                }
                continue;
            } else {
                conversation.add_astra_turn(&full_content);
                return full_content;
            }
        }

        break;
    }

    String::new()
}
