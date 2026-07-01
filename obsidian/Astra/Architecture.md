---
created: 2026-06-13
updated: 2026-07-01
---

# Architecture

[[ASTRA|← Home]]

## Overview

Astra is a Rust/Axum WebSocket server running on port `3000`. It bridges WebSocket clients to a local Ollama instance, maintaining per-connection conversation state and dispatching tool calls returned by the model.

## Request Flow

```
WebSocket client
    └── Axum router (src/main.rs)
            └── GET /ws → WebSocket upgrade
                    ↓
            handlers::ws::handle_socket (per-connection async loop)
                    ├── Parses incoming Envelope (backend/protocol.rs)
                    ├── Maintains conversation via Conversation (backend/conversation.rs)
                    │       └── Sliding window token management
                    └── run_agent_loop
                            └── backend::ollama::client (HTTP to Ollama /api/chat)
                                    ↓
                            model response:
                              tool_calls? → tools::dispatch → tools::implementations → continue loop
                              text?       → stream TextChunk envelopes back to client
```

## Modules

| Module | Responsibility |
|---|---|
| `src/main.rs` | Router setup, server startup, AppState init, `tracing_subscriber` logging init |
| `src/backend/state.rs` | `AppState` (read-only fields shared via `Arc`) — `Arc<Vec<Tool>>`, `Arc<str>` system prompt, Ollama URL + model, reqwest client (shared across all integrations), `Arc<WhisperContext>`, `Arc<Mutex<TtsEngine>>` (qwen3-tts), `Arc<VoiceFile>` speaker, `Arc<Mutex<String>>` Spotify access token, `Arc<Mutex<HashMap<String,String>>>` Spotify devices cache (`name → id`), `Option<Arc<HaClient>>` Home Assistant client (None if HA unreachable at startup) |
| `src/backend/config.rs` | Loads `.astra/core/*.md` + `.astra/user/*.md` into system prompt; parses `astra.conf` once into a map. Core keys: `OLLAMA_ENDPOINT`, `OLLAMA_MODEL`, `WHISPER_MODEL`, `TTS_VOICE`, `TTS_MAX_TOKENS`, `TTS_QUANT`. Integration-specific keys live in each integration's own `config.rs`. |
| `src/backend/protocol.rs` | WebSocket message schema — `Envelope`, `Message` enum, payload structs |
| `src/backend/conversation.rs` | Per-connection message history with sliding window enforcement |
| `src/backend/ollama/client.rs` | reqwest wrapper around Ollama `/api/chat` |
| `src/backend/ollama/types.rs` | `OllamaMessage`, `ChatRequest`, `Role` enum |
| `src/backend/audio.rs` | Audio module root |
| `src/backend/audio/stt.rs` | STT — `load_whisper_ctx` + `transcribe` (whisper-rs 0.16.0, `spawn_blocking`) |
| `src/backend/audio/tts.rs` | TTS — `load_tts_model` + `synthesize_sentence_streaming` (qwen3-tts `TtsEngine`, `spawn_blocking`, streams PCM chunks via a `tokio::mpsc` channel); `strip_markdown` + `take_sentences` pre-process LLM output |
| `src/handlers/ws.rs` | WebSocket upgrade handler, per-connection loop, agent loop, audio buffer, full voice pipeline |
| `src/tools/registry.rs` | `Tool`/`ToolFunction` structs, `register_tools()` |
| `src/tools/dispatch.rs` | Routes tool call by name to implementation; accepts `&AppState` for integration access |
| `src/tools/implementations.rs` | Async tool logic (`tokio::process::Command` for shell tools; reads `AppState` fields for integration tools) |
| `src/integrations.rs` | Integrations module root |
| `src/integrations/web.rs` | Web integrations sub-module root |
| `src/integrations/web/ddg_search.rs` | DuckDuckGo search (stub) |
| `src/integrations/spotify.rs` | Spotify sub-module root |
| `src/integrations/spotify/spotify_connection.rs` | Spotify API — `refresh_access_token`, `get_devices`, `play`, `pause`, `resume`, `search`; pure functions, no `AppState` dependency |
| `src/integrations/home.rs` | Home integrations sub-module root |
| `src/integrations/home/ha.rs` | HA sub-module root |
| `src/integrations/home/ha/home_assistant.rs` | `HaClient` struct — `Arc<Mutex<SplitSink>>` sink, `AtomicU32` message ID counter, `Arc<Mutex<HashMap<u32, oneshot::Sender<Value>>>>` pending map; `connect()` + `establish()` (shared auth handshake) + `reconnect()` (swaps sink, resets counter + pending, spawns new event_loop); private `send_command(payload)` abstracts all HA WS plumbing; public methods: `get_states`, `get_entity_registry`, `get_area_registry`, `get_devices`, `call_service` |
| `src/integrations/home/ha/types.rs` | `HaDevice` struct — `entity_id`, `friendly_name`, `aliases: Vec<String>`, `area: Option<String>`, `state`; derives `Serialize` + `Debug` |
| `src/integrations/home/ha/config.rs` | HA config getters — `ha_url()`, `ha_token()` (keys: `HOME_ASSISTANT_ENDPOINT`, `HOME_ASSISTANT_TOKEN`) |

## Conversation State

Ollama is fully stateless — it has no memory between API calls. Astra owns the `messages` array for each active session and replays the full history on every request to Ollama. The system prompt is prepended to this array on every call.

To prevent unbounded growth, Astra applies a **sliding window**: always keep the system prompt + the last N message turns, dropping the oldest when the limit is reached. N is computed at startup as `max_context_tokens - system_prompt_tokens - response_buffer` — the system prompt is always guaranteed to fit; conversation history fills the remaining space.

Current storage: **in-memory per session** (lost on server restart). Persistence deferred.

## Model Configuration

Behavioral config is split into two layers, assembled at startup into a single system prompt injected into every Ollama request:

```
.astra/
  astra.conf            # runtime config (OLLAMA_IP, etc.)
  core/
    identity.md         # what Astra is, what it can do
    tools.md            # rules around tool use
    behavior.md         # hard constraints
  user/
    personality.md      # name, tone, persona
    collaboration.md    # how it works with the user
    coding-style.md     # preferences when writing code
```

Core is loaded first, user config appended after. Core uses hard constraint language; user config covers softer preferences. User files can be absent without error. The combined token count determines the sliding window size N. `astra.conf` is a `KEY=VALUE` file parsed at startup for runtime settings like the Ollama server URL.

Hardware/runtime tuning (`num_ctx`, GPU offload, thread count) is managed via an Ollama Modelfile on the host machine and is outside Astra's scope.

## WebSocket Protocol

All text frames use a JSON envelope:

```json
{ "type": "<message_type>", "request_id": "<optional>", "payload": { ... } }
```

| Type | Direction | Notes |
|---|---|---|
| `text_message` | Client → Server | Text chat input; optional `voice_response: bool` triggers TTS reply |
| `text_chunk` | Server → Client | Streaming LLM output; `done: true` = end of response |
| `tool_call` | Server → Client | Tool dispatched by model |
| `tool_result` | Server → Client | Tool output sent back to model |
| `audio_end` | Client → Server | Signals end of mic audio (push-to-talk) |
| `transcript` | Server → Client | STT result from Whisper |
| `tts_start` | Server → Client | Start of TTS audio stream; carries `TtsStartPayload { sample_rate, channels, format }` so the client can configure playback before the first chunk |
| `tts_sentence` | Server → Client | Marks the sentence whose audio chunks follow; carries `TtsSentencePayload { text }` (spoken text) so the client reveals the transcript in sync with the audio |
| `tts_end` | Server → Client | Signals end of TTS audio stream; carries `TtsEndPayload { sample_rate, channels, format }` |
| `error` | Server → Client | Error envelope |

**Binary frames** carry raw PCM audio — no JSON wrapper. Incoming = mic audio (16kHz mono 16-bit LE); outgoing = TTS output (f32 LE, sample rate reported in `TtsEndPayload.sample_rate`).

## Audio Pipeline (Phase 2 — working, realtime)

```
binary WS frames (PCM chunks)
    └── handle_socket detects binary frame
            └── accumulate in audio buffer until AudioEnd JSON message
                    └── backend::audio::stt (whisper-rs / whisper.cpp FFI)
                            └── transcript text → run_agent_loop
                                    └── LLM response text (streamed)
                                            └── per sentence: backend::audio::tts (qwen3-tts)
                                                    └── TtsStart → PCM binary frame(s) → TtsEnd
```

**STT:** `whisper-rs` v0.16.0 — whisper.cpp via C FFI (CUDA). Requires LLVM/libclang on Windows for the bindgen build step.
**TTS:** the [`Qwen3-TTS-Rust`](https://github.com/cgisky1980/Qwen3-TTS-Rust) crate — Qwen3-TTS as GGUF via llama.cpp (Vulkan) for the talker/predictor + an ONNX audio decoder (CPU on Linux). Stored as `Arc<Mutex<TtsEngine>>` (synthesis takes `&mut self`); the speaker is an `Arc<VoiceFile>` loaded once. Output is f32 PCM at 24 kHz (`TTS_SAMPLE_RATE` const). Native libs are dlopen'd from `./runtime` (process CWD); model files in `.astra/models/tts/`, speaker JSONs in `.astra/models/tts/speakers/`.

**Latency:** **RTF ~0.80 on the Linux server (RTX 5060 Ti, Vulkan)** — under realtime, coexisting with whisper (CUDA) + the resident Ollama 9B (CUDA) on the same GPU. Synthesis is per-sentence (`take_sentences` + `speak_sentence`); each sentence streams sub-sentence as the PCM chunks the decoder emits (~every 320 ms via `generate_with_voice_streaming`), preceded by a `tts_sentence` marker so the web client reveals the transcript in step with the audio. **Must be built `--release`** — a debug build runs ~10× slower (RTF ~8) because the crate's per-frame Rust starves the GPU; `[profile.dev.package."*"] opt-level = 3` mitigates dev builds. Instrumentation logs per-synth RTF + LLM first-token timing. Knob: `TTS_QUANT` (q5_k_m saves ~1.7 GB VRAM). See [[Decisions]] (2026-06-25).

## Current Tools

| Tool | Description |
|---|---|
| `echo_hello_world` | Echoes "Hello, World!" via shell command |
| `list_contents` | Lists working directory contents |
| `spotify_get_devices` | Returns cached list of active Spotify device names |
| `spotify_search` | Searches Spotify for tracks, albums, playlists — returns `(name, uri)` list for model to choose from |
| `spotify_play_content` | Plays a Spotify URI on a named or active device |
| `spotify_pause_content` | Pauses playback on the active device |
| `spotify_resume_content` | Resumes playback on the active device |
| `ha_get_devices` | Returns `Vec<HaDevice>` — three-way join of HA state + entity registry + area registry, filtered to `should_expose == true` entities |
| `ha_toggle_device` | Calls `homeassistant.toggle` on a given `entity_id` |
| `ha_reconnect` | Re-establishes the HA WebSocket connection without restarting the server |
