---
created: 2026-06-13
updated: 2026-06-25
---

# Todos

[[ASTRA|← Home]]

## In Progress

- [ ] Full mic → STT → LLM → TTS round-trip test with the web client (blocked: mic capture needs a secure context — see Backlog)

## Backlog

### Phase 2 — Voice Pipeline

- [ ] Microphone capture fails over plain-HTTP LAN — `getUserMedia` needs a secure context (HTTPS or `localhost`); serve the web client over HTTPS (mkcert/tunnel) for LAN mic access
- [ ] Surface mic-capture errors in the web client — `startRecording`'s silent `catch` hides secure-context / permission / no-device failures (the button just looks dead)
- [ ] (Minor) Web client: guard the overlapping-response case where a new reply's `speakingId` can be cleared by the previous reply's last-chunk `onended`
- [ ] Interruption handling — user speaks while the assistant is responding
- [ ] (Deferred, only if needed) GPU ONNX decode — would need a crate fork to add a CUDA execution provider; unnecessary while RTF < 1

### Phase 3 — Tool Layer

- [ ] Refactor tool implementations into per-tool modules
- [ ] Structured tool error handling (tool failures must not crash the request)
- [ ] Define and implement first real tool

## Done

- [x] Sub-sentence TTS streaming — `synthesize_sentence_streaming` forwards PCM chunks (~320 ms) via a `tokio::mpsc` channel as the decoder emits them; web client plays them gapless via a Web Audio scheduling cursor (replaces buffer-then-play)
- [x] Transcript synced to audio — new `tts_sentence` protocol marker; web client reveals each sentence's text on the Web Audio clock as it's spoken
- [x] Web client (`astra-web`) streaming playback rework — `playback.ts` cursor, `ws.tsx` per-frame play + `tts_start`/`tts_sentence` handling, dropped the `tts_end` buffer/merge
- [x] TTS migrated to the `Qwen3-TTS-Rust` crate (GGUF/llama.cpp + ONNX) — `Arc<Mutex<TtsEngine>>`, async load, per-sentence synthesis; **RTF 0.80 on the server (realtime)**; replaced `any-tts` (see Decisions 2026-06-25)
- [x] Diagnosed the debug-build RTF-8 cliff → `--release` (RTF 0.80); added `[profile.dev.package."*"] opt-level = 3`; ruled out CUDA/Vulkan contention as a red herring
- [x] Validated whisper-rs (CUDA) + qwen3-tts (Vulkan) + resident Ollama 9B coexist on one GPU in one process
- [x] Provisioned TTS model/runtime/speakers (`.astra/models/tts/`, `./runtime`); `TTS_QUANT` knob replaces `TTS_DTYPE`/`TTS_MODEL_ID`
- [x] Whisper `ggml-base.en.bin` provisioned to `.astra/models/stt/`
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
