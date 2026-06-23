---
created: 2026-06-13
updated: 2026-06-22
---

# Progress

[[ASTRA|← Home]]

---

## 2026-06-22 — Phase 2 Audio Pipeline Design

- Designed audio pipeline architecture: server-side STT/TTS, in-process Rust libraries, raw PCM over binary WebSocket frames
- Evaluated Kokoro TTS crates; selected `any-tts` v0.1.1 (Candle-based, pure-Rust phonemizer, no system deps, trait-based API) over alternatives (`kokoroxide` requires espeak-ng, `tts-rs` unclear streaming support)
- Selected `whisper-rs` v0.16.0 (whisper.cpp via C FFI) for STT; rejected `faster-whisper-rs` (v0.1.0, 21 stars, calls Python API under the hood)
- Added `whisper-rs` and `any-tts` to `Cargo.toml`; resolved Windows build dependency: LLVM/libclang required for whisper-rs bindgen step (installed via winget)
- Added audio control message types to `backend/protocol.rs`: `AudioEnd`, `Transcript(TranscriptPayload)`, `TtsEnd`; binary WS frames carry raw PCM
- Push-to-talk selected for utterance boundary signaling; VAD deferred as a client-side concern
- Created `backend/audio/` module skeleton (`audio.rs`, `audio/stt.rs`, `audio/tts.rs`)
- Added binary WebSocket frame handling to `handlers/ws.rs`: `audio_buffer: Vec<u8>` accumulates PCM chunks; `AudioEnd` message triggers pipeline (stubbed); `WsFrame::Binary` arm appends to buffer
- Added `whisper_model_path: String` to `AppState`, loaded from `WHISPER_MODEL` key in `astra.conf`; added `load_whisper_model_path()` to `config.rs`
- Decided: `WhisperContext` will be stored as `Arc<WhisperContext>` (no Mutex) — model is read-only after loading, each connection creates its own `WhisperState` per transcription for true concurrent multi-user STT

---

## 2026-06-21 — Phase 2 Cleanup and Restructure

- Full code review (`/rust-review`) — 16 findings across critical, warnings, suggestions; all resolved
- Deleted dead HTTP handler layer (`req_handler.rs`); moved `OllamaMessage`, `ChatRequest`, `Role` enum into `backend/ollama/types.rs`
- Fixed async correctness: `std::process::Command` → `tokio::process::Command` in tool implementations
- Fixed hardcoded absolute macOS script path to relative path
- Added `reqwest::Client` to `AppState` (created once, reused); `ollama_url` now read from `.astra/astra.conf` via `load_ollama_url()`
- Restructured `src/` into `backend/` (state, config, conversation, protocol, ollama/), `handlers/`, `tools/`; migrated to Rust 2018+ named module file convention throughout
- Updated `CLAUDE.md` with accurate Architecture section and new Conventions section; rust-review skill updated to flag `mod.rs`

---

## 2026-06-22 — Phase 1 Complete

- Completed `src/conversation.rs`: all turn methods, sliding window, system prompt at index 0
- Created `src/config.rs`: loads and assembles `config/core/` + `config/user/` markdown files into system prompt at startup
- Populated all six config files (core: identity, behavior, tools; user: personality, collaboration, coding-style)
- Wired system prompt into `AppState` and `Conversation::new`
- Implemented streaming in `ws_handler`: NDJSON line buffering, token-by-token `text_chunk` delivery
- Implemented tool call agent loop: detects `tool_calls` in stream, dispatches via `dispatch_tool`, adds results to conversation history, re-enters Ollama call loop; breaks on text response
- Extended `OllamaMessage` with optional `tool_calls` field (`#[serde(skip_serializing_if = "Option::is_none")]`)
- Known limitation: tool implementations return `ExitStatus` not stdout — tool results sent to model are not useful yet (Phase 2 fix)

---

## 2026-06-21 — Phase 1 WebSocket Foundation (in progress)

- Designed and logged all major architectural decisions: Ollama statelessness, Astra owning conversation state, sliding window context management, two-layer config system (core/user), streaming routing strategy, WebSocket message schema
- Replaced HTTP routes with a single `/ws` WebSocket endpoint in `main.rs`
- Created `src/protocol.rs`: `Envelope` + `Message` enum (adjacently tagged via serde) with payload structs for all message types; audio deferred as binary frames
- Created `src/handlers/ws_handler.rs`: WebSocket upgrade handler and per-connection message loop stub
- Created `src/conversation.rs`: `Conversation` struct with history Vec, token estimator, and sliding window enforcement; `add_user_turn`/`add_assistant_turn`/`messages()` in progress
- Renamed `Message` → `OllamaMessage` in `req_handler.rs` to resolve naming collision with `protocol::Message`

---

## 2026-06-13 — Project Setup

- Initialized Rust project with Axum, reqwest, serde, tokio
- Built basic HTTP server on port 3000 with three routes: `/ollama`, `/generate`, `/chat`
- Implemented Ollama client (`list_models`, `generate`, `chat`)
- Built tool layer: registry, dispatch, and two placeholder implementations (`echo_hello_world`, `list_contents`)
- Configured Claude Code for the project:
  - `CLAUDE.md` created with architecture overview and collaboration guidelines
  - Permissions auto-allowed for `cargo *` and common shell ops
  - Post-edit `cargo check` hook configured
  - Plugins: context7, security-guidance, rust-analyzer-lsp
- Obsidian vault created at `obsidian/Astra/` with base note structure
