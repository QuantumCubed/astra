# Astra

A home AI server — local, low-latency, no cloud dependency. Runs on a home server and talks to a local [Ollama](https://ollama.com/) instance over WebSocket.

## What it does

Astra bridges WebSocket clients to a local Ollama LLM. It maintains per-connection conversation history, enforces a sliding context window, assembles a layered system prompt from config files, and dispatches tool calls returned by the model back to the client.

**End goals:** real-time voice interface, browser-based text/voice client, smart home integration, agentic task execution.

## Status

Phase 1 (WebSocket Foundation) is complete. Phase 2 (Voice Interface) is in progress — the full STT → LLM → TTS pipeline works; TTS uses Qwen3-TTS via `any-tts`, and its realtime latency is being tuned.

## Setup

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- A running [Ollama](https://ollama.com/) instance accessible from the host machine
- **Linux / WSL2 required** for builds that include `whisper-rs` and `any-tts`. The two crates have a C runtime conflict on Windows MSVC (LNK2038 `/MD` vs `/MT`, via `candle-kernels`/`esaxx-rs`) that has no linker workaround. All audio-enabled builds run on Linux or WSL2 (Ubuntu). CUDA works via WSL2's GPU passthrough.
- **Linux build deps:** `sudo apt install -y libasound2-dev` (ALSA, pulled in transitively by the audio crates).

### Configuration

All runtime config lives in `.astra/` at the project root.

**`.astra/astra.conf`** — required. Create this file before running the server:

```
OLLAMA_ENDPOINT=http://<your-ollama-host>:11434
WHISPER_MODEL=.astra/models/stt/ggml-base.en.bin
TTS_VOICE=ryan
TTS_MAX_TOKENS=512
# TTS_MODEL_ID=Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice   # optional model override (default: 1.7B)
# TTS_DTYPE=f16                                         # optional: bf16 (default) | f16 | f32
```

The server will fail at startup with a clear error if `OLLAMA_ENDPOINT` or `WHISPER_MODEL` are missing. `TTS_VOICE` defaults to `ryan` if absent; `TTS_MAX_TOKENS`, `TTS_MODEL_ID`, and `TTS_DTYPE` are optional TTS tuning knobs. Whisper model files (`.ggml` format) can be downloaded from [ggerganov/whisper.cpp on Hugging Face](https://huggingface.co/ggerganov/whisper.cpp). `ggml-base.en.bin` is a good starting point (~142MB). Qwen3-TTS model weights (~4.5 GB for the 1.7B) auto-download from HuggingFace on first run — no manual download needed.

**`.astra/core/`** — hard behavioral constraints injected into every Ollama request (identity, tool rules, behavior). Loaded first.

**`.astra/user/`** — soft preferences (persona, tone, collaboration style). Appended after core. Files can be absent without error.

The combined token count of all config files determines the sliding context window size: `max_context_tokens - system_prompt_tokens - response_buffer`.

Hardware/runtime tuning (`num_ctx`, GPU offload, thread count) belongs in an Ollama Modelfile on the host machine — not in this repo.

### Run

```bash
cargo run     # start server on port 3000
```

The server listens on `0.0.0.0:3000`.

## Architecture

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
                              tool_calls? → dispatch → implementations → continue loop
                              text?       → stream TextChunk envelopes back to client
```

### Key modules

| Module | Responsibility |
|---|---|
| `src/main.rs` | Router setup, server startup, `AppState` init |
| `src/backend/state.rs` | `AppState` — tools, system prompt, Ollama URL, shared reqwest client |
| `src/backend/config.rs` | Loads `.astra/core/` and `.astra/user/` into system prompt; parses `astra.conf` |
| `src/backend/protocol.rs` | WebSocket message schema — `Envelope`, `Message` enum, payload structs |
| `src/backend/conversation.rs` | Conversation history, token estimation, sliding window enforcement |
| `src/backend/ollama/client.rs` | reqwest wrapper for the Ollama `/api/chat` API |
| `src/backend/ollama/types.rs` | `OllamaMessage`, `ChatRequest`, `Role` enum |
| `src/handlers/ws.rs` | WebSocket upgrade handler, per-connection loop, agent loop |
| `src/tools/registry.rs` | Tool definitions, `register_tools()` |
| `src/tools/dispatch.rs` | Routes tool calls by name to implementations |
| `src/tools/implementations.rs` | Concrete tool logic (`tokio::process::Command`) |

## Message Protocol

All messages share a common envelope:

```json
{
  "type": "<message_type>",
  "request_id": "<client-generated uuid>",
  "payload": { ... }
}
```

`request_id` is generated by the client and echoed on all server messages for that request. It is optional on server-initiated messages with no associated request.

| Type | Direction | Payload |
|---|---|---|
| `text_message` | Client → Server | `{ "content": "..." }` |
| `text_chunk` | Server → Client | `{ "content": "...", "done": bool }` |
| `tool_call` | Server → Client | `{ "name": "...", "args": {...} }` |
| `tool_result` | Server → Client | `{ "name": "...", "result": "..." }` |
| `audio_end` | Client → Server | `{}` — signals end of mic audio (push-to-talk) |
| `transcript` | Server → Client | `{ "text": "..." }` — STT result from Whisper |
| `tts_start` | Server → Client | `{ "sample_rate": 24000, "channels": 1, "format": "f32le" }` — start of TTS audio stream, sent before the first PCM frame |
| `tts_end` | Server → Client | `{ "sample_rate": 24000, "channels": 1, "format": "f32le" }` — signals end of TTS audio stream |
| `error` | Server → Client | `{ "message": "...", "code": "..." }` |

`done: true` on a `text_chunk` means the response for that `request_id` is complete. The WebSocket connection stays open.

**Audio** is not a JSON message type. Raw PCM audio travels as binary WebSocket frames — no wrapper. Incoming mic audio is 16kHz mono 16-bit; outgoing TTS is 24kHz. JSON control messages (`audio_end`, `transcript`, `tts_end`) coordinate the pipeline.

## Commands

```bash
cargo build   # compile
cargo run     # start server on port 3000
cargo test    # run tests
cargo check   # fast type-check without linking
cargo clippy  # lint
```

## Adding a tool

1. Add an `async fn` in `src/tools/implementations.rs`.
2. Register a `Tool::new(...)` entry in `src/tools/registry.rs` → `register_tools()`.
3. Add a match arm in `src/tools/dispatch.rs` → `dispatch_tool`.

## Roadmap

| Phase | Goal | Status |
|---|---|---|
| 1 | WebSocket Foundation — transport, protocol, conversation state, config | Complete |
| 2 | Voice Interface — STT/TTS, audio frames, push-to-talk, TTS streaming | In progress |
| 3 | Tool Layer — real tools, structured error handling, per-tool modules | Backlog |
| 4 | Web Client — browser-based text/voice UI | Pending |
| 5 | Agents & Expansion — multi-step agents, smart home, mobile/desktop clients | Pending |

Full details in `obsidian/Astra/Roadmap.md`.
