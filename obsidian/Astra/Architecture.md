---
created: 2026-06-13
updated: 2026-06-23
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
| `src/main.rs` | Router setup, server startup, AppState init |
| `src/backend/state.rs` | `AppState` — tools, system prompt, Ollama URL, shared reqwest client, `Arc<WhisperContext>`, `Arc<dyn TtsModel>`, `tts_sample_rate`, `tts_voice` |
| `src/backend/config.rs` | Loads `.astra/core/*.md` + `.astra/user/*.md` into system prompt; reads `astra.conf` keys (`OLLAMA_ENDPOINT`, `WHISPER_MODEL`, `KOKORO_MODEL`, `KOKORO_VOICE`) |
| `src/backend/protocol.rs` | WebSocket message schema — `Envelope`, `Message` enum, payload structs |
| `src/backend/conversation.rs` | Per-connection message history with sliding window enforcement |
| `src/backend/ollama/client.rs` | reqwest wrapper around Ollama `/api/chat` |
| `src/backend/ollama/types.rs` | `OllamaMessage`, `ChatRequest`, `Role` enum |
| `src/backend/audio.rs` | Audio module root |
| `src/backend/audio/stt.rs` | STT — `load_whisper_ctx` + `transcribe` (whisper-rs 0.16.0, `spawn_blocking`) |
| `src/backend/audio/tts.rs` | TTS — `load_tts_model` + `synthesize` (any-tts/Kokoro, `spawn_blocking`) |
| `src/handlers/ws.rs` | WebSocket upgrade handler, per-connection loop, agent loop, audio buffer, full voice pipeline |
| `src/tools/registry.rs` | `Tool`/`ToolFunction` structs, `register_tools()` |
| `src/tools/dispatch.rs` | Routes tool call by name to implementation |
| `src/tools/implementations.rs` | Async tool logic (`tokio::process::Command`) |

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
| `tts_end` | Server → Client | Signals end of TTS audio stream; carries `TtsEndPayload { sample_rate, channels, format }` |
| `error` | Server → Client | Error envelope |

**Binary frames** carry raw PCM audio — no JSON wrapper. Incoming = mic audio (16kHz mono 16-bit LE); outgoing = TTS output (f32 LE, sample rate reported in `TtsEndPayload.sample_rate`).

## Audio Pipeline (Phase 2 — complete)

```
binary WS frames (PCM chunks)
    └── handle_socket detects binary frame
            └── accumulate in audio buffer until AudioEnd JSON message
                    └── backend::audio::stt (whisper-rs / whisper.cpp FFI)
                            └── transcript text → run_agent_loop (existing)
                                    └── LLM response text
                                            └── backend::audio::tts (any-tts / Kokoro)
                                                    └── stream PCM chunks as binary WS frames → client
```

**STT:** `whisper-rs` v0.16.0 — whisper.cpp via C FFI. Requires LLVM/libclang on Windows for the bindgen build step.
**TTS:** `any-tts` v0.1.1 — Kokoro 82M via Candle (pure Rust, no system deps). Returns `Vec<f32>` PCM at 24kHz.

## Current Tools

| Tool | Description |
|---|---|
| `echo_hello_world` | Echoes "Hello, World!" via shell command |
| `list_contents` | Lists working directory contents |
