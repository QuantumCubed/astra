use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket, ws::Message as WsFrame},
    response::Response,
};
use crate::{
    backend::{
        audio::{stt::transcribe, tts::{strip_markdown, synthesize_sentence_streaming, take_sentences, TTS_SAMPLE_RATE}},
        conversation::Conversation,
        ollama::{client, types::ChatRequest},
        protocol::{Envelope, ErrorPayload, Message, TextChunkPayload, TranscriptPayload, TtsEndPayload, TtsStartPayload},
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
                                        if let Ok(s) = serde_json::to_string(&reply) {
                                            let _ = socket.send(WsFrame::Text(s.into())).await;
                                        }

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

/// Strip markdown from one sentence, synthesize it, and stream the audio frame.
///
/// `TtsStart` (carrying the audio format) is sent lazily before the first chunk so
/// the client can configure playback; samples are also accumulated into `all_samples`
/// for the `debug_audio.raw` dump written once the turn ends. Called inline as each
/// sentence completes, so audio streams alongside the still-arriving text.
async fn speak_sentence(
    socket: &mut WebSocket,
    state: &AppState,
    request_id: &Option<String>,
    sentence: &str,
    tts_started: &mut bool,
    all_samples: &mut Vec<f32>,
) {
    let text = strip_markdown(sentence);
    if text.is_empty() {
        return;
    }

    if !*tts_started {
        let start = Envelope {
            request_id: request_id.clone(),
            message: Message::TtsStart(TtsStartPayload {
                sample_rate: TTS_SAMPLE_RATE,
                channels: 1,
                format: "f32le".to_string(),
            }),
        };
        if let Ok(s) = serde_json::to_string(&start) {
            let _ = socket.send(WsFrame::Text(s.into())).await;
        }
        *tts_started = true;
    }

    // Stream the sentence: forward each PCM chunk the moment the decoder emits it (~every
    // 320 ms), so audio starts playing long before the whole sentence is synthesized.
    let started = std::time::Instant::now();
    let (mut chunks, handle) =
        synthesize_sentence_streaming(state.tts.clone(), state.tts_voice.clone(), text);
    let mut n_samples = 0usize;
    while let Some(chunk) = chunks.recv().await {
        if chunk.is_empty() {
            continue;
        }
        n_samples += chunk.len();
        let bytes: Vec<u8> = chunk.iter().flat_map(|s| s.to_le_bytes()).collect();
        all_samples.extend_from_slice(&chunk);
        let _ = socket.send(WsFrame::Binary(bytes.into())).await;
    }

    match handle.await {
        Ok(Ok(())) => {
            let synth_s = started.elapsed().as_secs_f32();
            let audio_s = n_samples as f32 / TTS_SAMPLE_RATE as f32;
            tracing::info!(
                "TTS: {audio_s:.1}s audio in {synth_s:.1}s (RTF {:.2})",
                synth_s / audio_s.max(0.01)
            );
        }
        Ok(Err(e)) => {
            send_error(socket, request_id.clone(), "TTS_ERROR", &format!("TTS failed: {e}")).await;
        }
        Err(e) => {
            send_error(socket, request_id.clone(), "TTS_ERROR", &format!("TTS task failed: {e}")).await;
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
    // TTS streaming state, carried across tool-call iterations so the audio spans the
    // whole turn. We synthesize each sentence the moment it completes (inline), so
    // audio for earlier sentences plays while later text is still streaming in.
    let mut unspoken = String::new();
    let mut tts_started = false;
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let req = ChatRequest {
            model: "qwen3.5:9b".to_string(),
            stream: true,
            messages: conversation.messages().to_vec(),
            tools: state.tools.clone(),
        };

        let llm_started = std::time::Instant::now();
        let chat = client::chat(&state.client, &state.ollama_url, req).await;
        if let Err(e) = &chat {
            tracing::error!("Ollama request failed: {e:?}");
            send_error(
                socket,
                request_id.clone(),
                "LLM_ERROR",
                &format!("Ollama request failed: {e}"),
            )
            .await;
            break;
        }
        if let Ok(res) = chat {
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

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(calls) = json["message"]["tool_calls"].as_array() {
                            tool_calls.extend(calls.iter().cloned());
                        }

                        let token = json["message"]["content"].as_str().unwrap_or("");
                        let done = json["done"].as_bool().unwrap_or(false);

                        if first_token && !token.is_empty() {
                            tracing::info!(
                                "LLM first token in {:.1}s",
                                llm_started.elapsed().as_secs_f32()
                            );
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
                            if let Ok(json_str) = serde_json::to_string(&reply) {
                                let _ = socket.send(WsFrame::Text(json_str.into())).await;
                            }
                        }

                        // Interleave: speak each sentence as soon as it forms. Synthesis
                        // blocks reading the next token, but ollama buffers in the
                        // meantime, so no text is lost — it just catches up after.
                        if voice_response && !token.is_empty() {
                            unspoken.push_str(token);
                            for sentence in take_sentences(&mut unspoken) {
                                speak_sentence(
                                    socket,
                                    state,
                                    &request_id,
                                    &sentence,
                                    &mut tts_started,
                                    &mut all_samples,
                                )
                                .await;
                            }
                        }
                    }
                }
            }

            tracing::info!(
                "LLM streamed {} chars in {:.1}s",
                full_content.len(),
                llm_started.elapsed().as_secs_f32()
            );

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
                break;
            }
        }

        break;
    }

    if !voice_response {
        return;
    }

    // Speak any trailing partial sentence (the final chunk often has no terminator),
    // then close the audio stream and dump the full utterance for offline inspection.
    let tail = unspoken.trim().to_string();
    if !tail.is_empty() {
        speak_sentence(
            socket,
            state,
            &request_id,
            &tail,
            &mut tts_started,
            &mut all_samples,
        )
        .await;
    }

    if tts_started {
        // ffplay -f f32le -ar <sample_rate> -ac 1 debug_audio.raw
        let dump: Vec<u8> = all_samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        let _ = std::fs::write("debug_audio.raw", &dump);

        let end = Envelope {
            request_id: request_id.clone(),
            message: Message::TtsEnd(TtsEndPayload {
                sample_rate: TTS_SAMPLE_RATE,
                channels: 1,
                format: "f32le".to_string(),
            }),
        };
        if let Ok(s) = serde_json::to_string(&end) {
            let _ = socket.send(WsFrame::Text(s.into())).await;
        }
    }
}
