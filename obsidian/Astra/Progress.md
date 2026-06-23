---
created: 2026-06-13
updated: 2026-06-23
---

# Progress

[[ASTRA|← Home]]

---

## 2026-06-23 — TTS Library Migration Complete (kokoro-tiny)

- Tested TTS output with `any-tts`; identified mispronounced words ("astra", "assistant", "object" as heteronym) — root cause: in-tree pure-Rust phonemizer has limited dictionary and no POS awareness
- Added `strip_markdown()` to `tts.rs` — strips `**`, `*`, `_`, `` ` ``, `#`, `>`, `- `, `1. `, `[text](url)` and code fences before phonemization; eliminates symbol noise in speech output
- Attempted `kokoroxide v0.1.5` — broken: all `ort ^1.16` versions yanked; crate unmaintained
- Attempted `tts-rs v2026.2.1` — broken against all available `ort` rc versions: rc.10 has `session.inputs` as a field (tts-rs calls it as a method); rc.12 changed `SessionBuilder` error types (open issue #1, maintainer unresponsive)
- Switched to `kokoro-tiny v0.1.0`: native async `TtsEngine::new()`, espeak-ng phonemizer (bundled statically via `espeak-rs-sys`, no system install), `cuda` feature, auto-downloads models to `~/.cache/k/`
- Rewrote `tts.rs`: `load_tts_engine()` (async, returns `Arc<Mutex<TtsEngine>>`), `synthesize()` (offloads to `spawn_blocking`), `strip_markdown()`
- Rewrote `state.rs`: `AppState::new()` is now `async fn`; `tts: Arc<Mutex<TtsEngine>>`; whisper loading still in its own `spawn_blocking`; removed `tts_sample_rate` field
- Updated `main.rs`: dropped `spawn_blocking` wrapper — `AppState::new().await` directly
- Simplified `config.rs`: removed `load_tts_model_path()`, `load_tts_voices_path()`, `load_kokoro_tokenizer_path()`; only `OLLAMA_ENDPOINT`, `WHISPER_MODEL`, `KOKORO_VOICE` keys remain
- Updated `Cargo.toml`: removed `kokoroxide`, `ort` pin; added `kokoro-tiny = { version = "0.1.0", features = ["cuda"] }`
- Updated `.astra/astra.conf`: removed `KOKORO_MODEL` and `KOKORO_TOKENIZER` keys; `KOKORO_VOICE=af_heart` retained
- Added webfetch reliability rule to `.claude/rules/webfetch.md` — prefer raw source over READMEs; treat WebFetch signatures as hypotheses; compiler errors override fetched claims
- Updated memory: `project_tts_library.md` documents full TTS crate landscape with reasons for each decision

---

## 2026-06-23 — Phase 2 Audio Pipeline Complete

- Implemented `backend/audio/tts.rs`: `load_tts_model()` (loads Kokoro via any-tts, returns `Arc<dyn TtsModel>`) and `synthesize()` (offloads to `spawn_blocking`, returns `Vec<f32>` PCM)
- Implemented `backend/audio/stt.rs`: `load_whisper_ctx()` (returns `Arc<WhisperContext>`) and `transcribe()` (offloads to `spawn_blocking`; converts 16-bit LE PCM to f32, runs Whisper inference, collects transcript via `state.as_iter()`)
- Updated `AppState` to store `whisper_ctx`, `tts_model`, `tts_sample_rate`, and `tts_voice`; all loaded at startup
- Added `KOKORO_MODEL` and `WHISPER_MODEL` keys to `.astra/astra.conf`; `KOKORO_VOICE` key added for optional voice selection (defaults to `af_heart`)
- Added `load_tts_model_path()`, `load_whisper_model_path()`, and `load_tts_voice()` to `config.rs`
- Added `anyhow = "1"` to `Cargo.toml` for cross-FFI error handling
- Wired full audio pipeline in `handlers/ws.rs`: `AudioEnd` → STT → send `Transcript` → add to conversation → run agent loop → TTS → send binary PCM frames (f32 LE) → send `TtsEnd(TtsEndPayload)`
- Added `send_error` helper; changed `run_agent_loop` to return `String` (final LLM response text for TTS input)
- Added `voice_response: bool` to `TextMessagePayload` — text messages can now also trigger TTS output
- Added `TtsEndPayload { sample_rate, channels, format }` to `TtsEnd` message — client no longer has to guess audio format
- Fixed whisper-rs 0.16.0 API mismatch: `full_n_segments()` returns `i32` (not `Result`); `full_get_segment_text` doesn't exist — replaced with `state.as_iter()` + `seg.to_str_lossy()`
- Added `WhisperContextParameters::use_gpu(true)` to STT loader — GPU is not auto-selected by whisper-rs even with `cuda` feature enabled
- Enabled `cuda` feature for both `whisper-rs` and `any-tts` in `Cargo.toml` (Linux/WSL2 only)
- Diagnosed fatal Windows MSVC CRT conflict (LNK2038): whisper-rs-sys compiles with `/MD`, esaxx-rs and candle-kernels with `/MT` — no linker workaround exists; established WSL2 (Ubuntu) as primary development environment for Linux/CUDA builds
- Fixed Tokio runtime panic at startup: `AppState::new()` moved into `tokio::task::spawn_blocking` in `main.rs` — any-tts's `load_model` creates an internal Tokio runtime that panics if dropped inside an async context
- Created `obsidian/Astra/Learning/audio-pipeline.md` — full walkthrough of tts.rs, stt.rs, ws.rs rewire with data flow diagram

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
