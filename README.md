# Astra

A home AI server — local, low-latency, no cloud dependency. Runs on a home server and talks to a local [Ollama](https://ollama.com/) instance over WebSocket.

## What it does

Astra bridges WebSocket clients to a local Ollama LLM. It maintains per-connection conversation history, enforces a sliding context window, assembles a layered system prompt from config files, and dispatches tool calls returned by the model back to the client.

**End goals:** real-time voice interface, browser-based text/voice client, smart home integration, agentic task execution.

## Status

Phase 1 (WebSocket Foundation) is complete. Phase 2 (Voice Interface) is largely working — the full STT → LLM → TTS pipeline runs realtime (RTF ~0.8 on the server). TTS uses Qwen3-TTS via the [`Qwen3-TTS-Rust`](https://github.com/cgisky1980/Qwen3-TTS-Rust) crate (GGUF/llama.cpp + ONNX).

## Setup

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- A running [Ollama](https://ollama.com/) instance accessible from the host machine
- **Linux (or WSL2) for builds and deployment.** STT (`whisper-rs`/whisper.cpp, CUDA) and TTS (the Qwen3-TTS-Rust crate's dlopen'd llama.cpp + onnxruntime, Vulkan) target Linux; the `runtime/` shared libraries are Linux `.so`s. CUDA works natively on Linux and via WSL2 GPU passthrough. (The historical Windows-MSVC blocker was the `any-tts`/candle `/MD` vs `/MT` CRT conflict; any-tts has since been removed.)
- **Linux build deps:** `sudo apt install -y libasound2-dev` (ALSA, pulled in transitively by the audio crates).

### Configuration

All runtime config lives in `.astra/` at the project root.

**`.astra/astra.conf`** — required. Create this file before running the server:

```
OLLAMA_ENDPOINT=http://<your-ollama-host>:11434
WHISPER_MODEL=.astra/models/stt/ggml-base.en.bin
TTS_VOICE=ryan                                          # speaker: .astra/models/tts/speakers/<name>.json
TTS_MAX_TOKENS=512                                      # cap on generation steps (codec frames) per sentence
TTS_QUANT=none                                          # GGUF quant: none | q5_k_m | q8_0 (q5_k_m saves ~1.7GB VRAM)
```

The server will fail at startup with a clear error if `OLLAMA_ENDPOINT` or `WHISPER_MODEL` are missing. `TTS_VOICE` defaults to `ryan` if absent; `TTS_MAX_TOKENS` and `TTS_QUANT` are optional TTS tuning knobs. Whisper model files (`.ggml` format) come from [ggerganov/whisper.cpp on Hugging Face](https://huggingface.co/ggerganov/whisper.cpp); `ggml-base.en.bin` is a good starting point (~142MB). For TTS, the Qwen3-TTS model files live in `.astra/models/tts/` (`gguf/`, `onnx/`, `tokenizer/` subdirs), speaker profiles in `.astra/models/tts/speakers/<name>.json`, and the dlopen'd llama.cpp + onnxruntime shared libraries in `./runtime/` at the project root (resolved relative to the working directory). **Run with `cargo run --release`** — a debug build is ~10× slower for TTS.

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
| 2 | Voice Interface — STT/TTS, audio frames, push-to-talk, TTS streaming | In progress (pipeline realtime; sub-sentence streaming next) |
| 3 | Tool Layer — real tools, structured error handling, per-tool modules | Backlog |
| 4 | Web Client — browser-based text/voice UI | Pending |
| 5 | Agents & Expansion — multi-step agents, smart home, mobile/desktop clients | Pending |

Full details in `obsidian/Astra/Roadmap.md`.
