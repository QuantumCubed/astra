---
created: 2026-06-13
updated: 2026-06-24
---

# Todos

[[ASTRA|← Home]]

## In Progress

- [ ] TTS latency: get RTF < 1 on the dev GPU — `TTS_DTYPE=f16` A/B, then a faster / non-autoregressive model

## Backlog

### Phase 2 — Voice Pipeline

- [ ] Download Whisper `ggml-base.en.bin` model file to `.astra/models/stt/`
- [ ] End-to-end voice path test with a real client
- [ ] Re-enable per-sentence interleaving once RTF < 1 (machinery kept in `tts.rs`/`ws.rs`)
- [ ] Investigate why `Qwen3-TTS-12Hz-0.6B-CustomVoice` failed to load/run (reverted to 1.7B)
- [ ] If RTF stays > 1: evaluate a non-autoregressive backend (Kokoro, RTF ~0.05)
- [ ] (Future) Fork any-tts to expose a streaming synthesis API for Qwen3-TTS

### Phase 3 — Tool Layer

- [ ] Refactor tool implementations into per-tool modules
- [ ] Structured tool error handling (tool failures must not crash the request)
- [ ] Define and implement first real tool

## Done

- [x] TTS migrated to `any-tts v0.1.2` / `ModelType::Qwen3Tts` — `Arc<dyn TtsModel>`, `spawn_blocking` load, BF16; replaced `kokoro-tiny` (see Decisions 2026-06-24)
- [x] Added runtime TTS knobs: `TTS_VOICE` (renamed from `KOKORO_VOICE`), `TTS_MAX_TOKENS`, `TTS_MODEL_ID`, `TTS_DTYPE`
- [x] Added `TtsStart(TtsStartPayload)` protocol message — audio format sent before the first chunk
- [x] Added observability: `tracing_subscriber`, model-load timing, per-synth RTF + LLM first-token logging
- [x] TTS library finalised: `kokoro-tiny v0.1.0` — `tts.rs`, `state.rs`, `config.rs`, `Cargo.toml`, `astra.conf` all updated; kokoroxide and tts-rs abandoned (see Decisions)
- [x] `AppState::new()` made async; `spawn_blocking` wrapper in `main.rs` removed — kokoro-tiny uses native async, no nested runtime issue
- [x] WSL2 (Ubuntu) established as development environment for Linux/CUDA builds — native Windows MSVC is blocked by an irresolvable CRT conflict (LNK2038)
- [x] `voice_response: bool` added to `TextMessagePayload` — text messages can optionally request TTS output
- [x] `TtsEnd` upgraded to `TtsEnd(TtsEndPayload)` with `sample_rate`, `channels`, `format` — client can configure audio playback correctly
- [x] `KOKORO_VOICE` key added to `astra.conf`; `load_tts_voice()` added to `config.rs`; `tts_voice: String` added to `AppState`
- [x] `WhisperContextParameters::use_gpu(true)` set in `load_whisper_ctx()` — GPU not auto-selected without explicit flag
- [x] `cuda` feature enabled for `whisper-rs` and `any-tts` in `Cargo.toml`
- [x] Add `Arc<WhisperContext>` and `Arc<dyn TtsModel>` to `AppState` — models loaded at startup, shared across connections
- [x] Implement STT in `backend/audio/stt.rs` — `load_whisper_ctx` + `transcribe` (whisper-rs 0.16.0, `as_iter()` API)
- [x] Implement TTS in `backend/audio/tts.rs` — `load_tts_model` + `synthesize` (any-tts/Kokoro)
- [x] Wire audio pipeline in `handlers/ws.rs` — `AudioEnd` → STT → `Transcript` → agent loop → TTS → binary PCM frames → `TtsEnd`
- [x] Add `KOKORO_MODEL` and `WHISPER_MODEL` keys to `.astra/astra.conf`; add `load_tts_model_path()` to `config.rs`
- [x] `backend/audio/` module skeleton created (`audio.rs`, `audio/stt.rs`, `audio/tts.rs`)
- [x] Binary WebSocket frame handling in `handlers/ws.rs` — PCM chunks accumulate in `audio_buffer`, `AudioEnd` triggers pipeline (stub)
- [x] `whisper_model_path` added to `AppState`, loaded from `WHISPER_MODEL` in `astra.conf`
- [x] STT/TTS placement decision — server-side, in-process (`whisper-rs` + `any-tts`); see [[Decisions]]
- [x] Audio protocol message types added to `backend/protocol.rs` (`AudioEnd`, `Transcript`, `TtsEnd`)
- [x] `whisper-rs` and `any-tts` added to `Cargo.toml`
- [x] Fix tool implementations to capture stdout — `.status()` replaced with `.output()`, cross-platform via `#[cfg]`
- [x] Phase 2 cleanup — dead code removed (`req_handler.rs` deleted, `OllamaMessage`/`ChatRequest` moved to `backend/ollama/types.rs`)
- [x] Codebase restructured into `backend/`, `handlers/`, `tools/` modules
- [x] Ollama URL moved from hardcoded constant to `.astra/astra.conf`
- [x] `reqwest::Client` created once in `AppState`, reused across requests
- [x] `role: String` replaced with typed `Role` enum in `OllamaMessage`
- [x] Async correctness — `std::process::Command` replaced with `tokio::process::Command`
- [x] Hardcoded absolute script path fixed to relative path
- [x] Modern Rust 2018+ module convention enforced (named files, no `mod.rs`)
- [x] WebSocket message schema (`src/protocol.rs` — Envelope, Message enum, payload structs)
- [x] WebSocket route and handler (`src/handlers/ws_handler.rs`)
- [x] Conversation state with sliding window (`src/conversation.rs`)
- [x] System prompt config — two-layer core/user markdown files, loaded at startup via `src/config.rs`
- [x] Streaming Ollama responses token-by-token over WebSocket
- [x] Tool call detection and agent loop (detect tool_calls in stream, dispatch, re-enter loop)
- [x] Basic Axum server with `/ollama`, `/generate`, `/chat` routes (retired)
- [x] Ollama client for `list_models`, `generate`, `chat`
- [x] Tool registry, dispatch, and implementation layer
- [x] Claude Code configured (permissions, cargo check hook, plugins)
- [x] Obsidian vault set up
