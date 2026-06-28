use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket, ws::Message as WsFrame},
    response::Response,
};
use crate::{
    backend::{
        audio::{stt::transcribe, tts::{strip_markdown, synthesize_sentence_streaming, take_sentences, TTS_SAMPLE_RATE}},
        conversation::Conversation,
        ollama::{client, types::ChatRequest},
        protocol::{Envelope, ErrorPayload, Message, TextChunkPayload, TranscriptPayload, TtsEndPayload, TtsSentencePayload, TtsStartPayload},
        state::AppState,
    },
    tools::dispatch,
};
use futures::StreamExt;

// Must be `async`: axum only implements `Handler` for functions returning a `Future`,
// even though `on_upgrade` itself returns synchronously (no `.await` needed here).
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut conversation = Conversation::new(&state.system_prompt);
    let mut audio_buffer: Vec<u8> = Vec::new();

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            WsFrame::Text(text) => {
                match serde_json::from_str::<Envelope>(&text) {
                    Ok(envelope) => {
                        let request_id = envelope.request_id.clone();
                        match envelope.message {
                            Message::TextMessage(payload) => {
                                tracing::info!(
                                    "text_message received (voice_response={})",
                                    payload.voice_response
                                );
                                conversation.add_user_turn(&payload.content);
                                run_agent_loop(
                                    &mut socket,
                                    &state,
                                    &mut conversation,
                                    request_id.clone(),
                                    payload.voice_response,
                                )
                                .await;
                            }
                            Message::AudioEnd => {
                                let audio = std::mem::take(&mut audio_buffer);
                                tracing::info!("audio_end received ({} bytes)", audio.len());
                                match transcribe(state.whisper_ctx.clone(), audio).await {
                                    Ok(transcript) => {
                                        let reply = Envelope {
                                            request_id: request_id.clone(),
                                            message: Message::Transcript(TranscriptPayload {
                                                text: transcript.clone(),
                                            }),
                                        };
                                        send_frame(&mut socket, &reply).await;

                                        conversation.add_user_turn(&transcript);
                                        run_agent_loop(
                                            &mut socket,
                                            &state,
                                            &mut conversation,
                                            request_id.clone(),
                                            true,
                                        )
                                        .await;
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
                        tracing::warn!("failed to parse incoming message: {e}");
                        let reply = Envelope {
                            request_id: None,
                            message: Message::Error(ErrorPayload {
                                message: format!("invalid message format: {e}"),
                                code: "PARSE_ERROR".to_string(),
                            }),
                        };
                        send_frame(&mut socket, &reply).await;
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

/// Serialize and send one JSON envelope. Returns `false` if the socket send failed (the
/// client has likely disconnected), so callers in a loop can stop doing work no one will
/// receive. A serialization failure is logged and treated as non-fatal (`true`).
async fn send_frame(socket: &mut WebSocket, envelope: &Envelope) -> bool {
    match serde_json::to_string(envelope) {
        Ok(s) => socket.send(WsFrame::Text(s.into())).await.is_ok(),
        Err(e) => {
            tracing::error!("failed to serialize outgoing frame: {e}");
            true
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
    send_frame(socket, &reply).await;
}

/// Per-turn TTS streaming state. Buffers streamed tokens into sentences, synthesizes and
/// streams each the moment it completes, and (at `finish`) speaks any trailing partial
/// sentence and closes the audio stream. Held across tool-call iterations so the audio
/// spans the whole turn. Keeping this separate from `run_agent_loop` keeps the LLM loop
/// and the speech lifecycle independent.
struct TtsStream {
    unspoken: String,
    started: bool,
    all_samples: Vec<f32>,
}

impl TtsStream {
    fn new() -> Self {
        Self {
            unspoken: String::new(),
            started: false,
            all_samples: Vec::new(),
        }
    }

    /// Feed a freshly-streamed token; speak any sentences it completes. Returns `false` if
    /// the client disconnected (the caller should stop the turn).
    async fn feed(
        &mut self,
        socket: &mut WebSocket,
        state: &AppState,
        request_id: &Option<String>,
        token: &str,
    ) -> bool {
        self.unspoken.push_str(token);
        for sentence in take_sentences(&mut self.unspoken) {
            if !self.speak(socket, state, request_id, &sentence).await {
                return false;
            }
        }
        true
    }

    /// Strip markdown from one sentence, synthesize it, and stream the PCM out frame by
    /// frame as the decoder emits it. `TtsStart` (audio format) is sent lazily before the
    /// first sentence; a `TtsSentence` marker (the spoken text) precedes each sentence's
    /// chunks so the client can reveal the transcript in sync. Returns `false` if a send
    /// failed (client gone); synthesis errors are reported but don't stop the turn.
    async fn speak(
        &mut self,
        socket: &mut WebSocket,
        state: &AppState,
        request_id: &Option<String>,
        sentence: &str,
    ) -> bool {
        let text = strip_markdown(sentence);
        if text.is_empty() {
            return true;
        }

        if !self.started {
            let start = Envelope {
                request_id: request_id.clone(),
                message: Message::TtsStart(TtsStartPayload {
                    sample_rate: TTS_SAMPLE_RATE,
                    channels: 1,
                    format: "f32le".to_string(),
                }),
            };
            if !send_frame(socket, &start).await {
                return false;
            }
            self.started = true;
        }

        let marker = Envelope {
            request_id: request_id.clone(),
            message: Message::TtsSentence(TtsSentencePayload { text: text.clone() }),
        };
        if !send_frame(socket, &marker).await {
            return false;
        }

        let started = std::time::Instant::now();
        let (mut chunks, handle) =
            synthesize_sentence_streaming(state.tts.clone(), state.tts_voice.clone(), text);
        let mut n_samples = 0usize;
        let mut n_chunks = 0usize;
        let mut first_chunk_s: Option<f32> = None;
        while let Some(chunk) = chunks.recv().await {
            if chunk.is_empty() {
                continue;
            }
            first_chunk_s.get_or_insert_with(|| started.elapsed().as_secs_f32());
            n_chunks += 1;
            n_samples += chunk.len();
            let bytes: Vec<u8> = chunk.iter().flat_map(|s| s.to_le_bytes()).collect();
            self.all_samples.extend_from_slice(&chunk);
            if socket.send(WsFrame::Binary(bytes.into())).await.is_err() {
                // Client gone — stop the turn. The in-flight synthesis detaches and finishes.
                return false;
            }
        }

        match handle.await {
            Ok(Ok(())) => {
                let synth_s = started.elapsed().as_secs_f32();
                let audio_s = n_samples as f32 / TTS_SAMPLE_RATE as f32;
                // n_chunks > 1 with a sub-second first chunk = genuinely streaming.
                tracing::info!(
                    "TTS: {audio_s:.1}s audio in {synth_s:.1}s (RTF {:.2}); {n_chunks} chunks, first at {:.2}s",
                    synth_s / audio_s.max(0.01),
                    first_chunk_s.unwrap_or(synth_s)
                );
            }
            Ok(Err(e)) => {
                send_error(socket, request_id.clone(), "TTS_ERROR", &format!("TTS failed: {e}")).await;
            }
            Err(e) => {
                send_error(socket, request_id.clone(), "TTS_ERROR", &format!("TTS task failed: {e}")).await;
            }
        }
        true
    }

    /// Speak any trailing partial sentence (the final token usually has no terminator) and
    /// close the audio stream with `TtsEnd`.
    async fn finish(mut self, socket: &mut WebSocket, state: &AppState, request_id: &Option<String>) {
        let tail = self.unspoken.trim().to_string();
        if !tail.is_empty() {
            let _ = self.speak(socket, state, request_id, &tail).await;
        }

        if self.started {
            // Debug aid: dump the whole utterance as raw f32 PCM for offline inspection
            // (`ffplay -f f32le -ar 24000 -ac 1 debug_audio.raw`). Disabled by default —
            // uncomment when working on the audio pipeline. Uses tokio::fs so it won't
            // block the async runtime.
            // let dump: Vec<u8> = self.all_samples.iter().flat_map(|s| s.to_le_bytes()).collect();
            // let _ = tokio::fs::write("debug_audio.raw", &dump).await;

            let end = Envelope {
                request_id: request_id.clone(),
                message: Message::TtsEnd(TtsEndPayload {
                    sample_rate: TTS_SAMPLE_RATE,
                    channels: 1,
                    format: "f32le".to_string(),
                }),
            };
            let _ = send_frame(socket, &end).await;
        }
    }
}

async fn run_agent_loop(
    socket: &mut WebSocket,
    state: &AppState,
    conversation: &mut Conversation,
    request_id: Option<String>,
    voice_response: bool,
) {
    let mut tts = TtsStream::new();

    loop {
        let req = ChatRequest {
            model: state.model.clone(),
            stream: true,
            messages: conversation.messages().to_vec(),
            tools: (*state.tools).clone(),
        };

        let llm_started = std::time::Instant::now();
        let res = match client::chat(&state.client, &state.ollama_url, req).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Ollama request failed: {e:?}");
                send_error(socket, request_id.clone(), "LLM_ERROR", &format!("Ollama request failed: {e}")).await;
                break;
            }
        };

        let mut stream = res.bytes_stream();
        let mut line_buf = String::new();
        let mut full_content = String::new();
        let mut tool_calls: Vec<serde_json::Value> = Vec::new();
        let mut first_token = true;

        while let Some(Ok(chunk)) = stream.next().await {
            line_buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(pos) = line_buf.find('\n') {
                let line: String = line_buf.drain(..=pos).collect();
                let line = line.trim();

                let Ok(json) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };

                if let Some(calls) = json["message"]["tool_calls"].as_array() {
                    tool_calls.extend(calls.iter().cloned());
                }

                let token = json["message"]["content"].as_str().unwrap_or("");
                let done = json["done"].as_bool().unwrap_or(false);

                if first_token && !token.is_empty() {
                    tracing::info!("LLM first token in {:.1}s", llm_started.elapsed().as_secs_f32());
                    first_token = false;
                }

                full_content.push_str(token);

                if !token.is_empty() || done {
                    let reply = Envelope {
                        request_id: request_id.clone(),
                        message: Message::TextChunk(TextChunkPayload {
                            content: token.to_string(),
                            done,
                        }),
                    };
                    // A failed send means the client is gone — stop the whole turn so we
                    // don't keep synthesizing speech nobody will receive.
                    if !send_frame(socket, &reply).await {
                        return;
                    }
                }

                // Interleave: speak each sentence as soon as it forms. Synthesis blocks
                // reading the next token, but ollama buffers in the meantime, so no text is
                // lost — it just catches up after.
                if voice_response
                    && !token.is_empty()
                    && !tts.feed(socket, state, &request_id, token).await
                {
                    return;
                }
            }
        }

        tracing::info!(
            "LLM streamed {} chars in {:.1}s",
            full_content.len(),
            llm_started.elapsed().as_secs_f32()
        );

        if tool_calls.is_empty() {
            conversation.add_astra_turn(&full_content);
            break;
        }

        conversation.add_astra_tool_call(tool_calls.clone());
        for tool_call in &tool_calls {
            let name = tool_call["function"]["name"].as_str().unwrap_or_default();
            let args = tool_call["function"]["arguments"].clone();
            let result = dispatch::dispatch_tool(name, args, state).await;
            let result_str =
                serde_json::to_string(&result).unwrap_or_else(|_| "error".to_string());
            conversation.add_tool_result(&result_str);
        }
    }

    if voice_response {
        tts.finish(socket, state, &request_id).await;
    }
}
