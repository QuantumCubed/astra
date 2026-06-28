---
created: 2026-06-13
updated: 2026-06-28
---

# Progress

[[ASTRA|← Home]]

---

## 2026-06-28 — Spotify playback tools: play, pause, resume, search

- Implemented `play`, `pause`, `resume` in `spotify_connection.rs` — all take `device_id: Option<&str>` and conditionally attach the query param; `play` routes `spotify:track:` URIs to `uris[]` and all others to `context_uri` via `.starts_with()`.
- Implemented `spotify_search` in `spotify_connection.rs` — `GET /v1/search` with `type=track,album,playlist&limit=5`; returns `Vec<(name, uri)>`; null items handled via `Vec<Option<SpotifySearchItem>>` + `.into_iter().flatten()`.
- Wired four new tools end-to-end: `spotify_search`, `spotify_play_content`, `spotify_pause_content`, `spotify_resume_content` — through `implementations.rs`, `dispatch.rs`, `registry.rs`. `spotify_play_content` looks up device name → ID from the devices cache and falls back to active device on miss.
- Two-step search pattern: model calls `spotify_search` to get `(name, uri)` results, picks the best match, then calls `spotify_play_content` with the URI. Keeps each tool single-purpose.
- Added `reqwest "query"` feature to `Cargo.toml` — `.query()` is not in reqwest 0.13's core.
- Verified `spotify_pause_content` and `spotify_resume_content` working; `spotify_search` fixed after discovering Spotify returns null items in `items` arrays.

---

## 2026-06-28 — Spotify integration foundation: auth, devices cache, first tool

- Set up Spotify OAuth manually (Authorization Code flow) — obtained refresh token via browser callback + Bruno POST; token stored in `.astra/astra.conf`.
- Added `SPOTIFY_CLIENT_ID`, `SPOTIFY_CLIENT_SECRET`, `SPOTIFY_REFRESH_TOKEN` to `astra.conf` and three reader functions to `backend/config.rs`.
- Extended `AppState` with `spotify_token: Arc<Mutex<String>>` and `spotify_devices: Arc<Mutex<HashMap<String,String>>>`. At startup: mint access token via `refresh_access_token`, then call `get_devices` to pre-populate the devices cache (`name → id`). Empty map on failure; refreshed on first tool error.
- Implemented `integrations/spotify/spotify_connection.rs`: `refresh_access_token` (form POST + Basic auth → bearer token) and `get_devices` (Bearer auth → `HashMap<name, id>`).
- Added `reqwest` `form` feature to `Cargo.toml`. Consolidated to a single shared `reqwest::Client` (moved out of inline `Self {}` into a pre-constructed local so it can be passed to `refresh_access_token` before being moved into the struct).
- Added `spotify_get_devices` tool end-to-end: `registry.rs` (no-args tool), `dispatch.rs` (now accepts `&AppState`), `implementations.rs` (locks `state.spotify_devices`, returns device names as serialized JSON). `ws.rs` updated to pass `&state` to `dispatch_tool`.
- Verified the model can call `spotify_get_devices` and receive the device list.
- Protected `.astra/astra.conf` in Claude Code settings (Read/Edit/Write denied — credentials file).

---

## 2026-06-25 — Streaming TTS + transcript synced to audio; web client playback

- **Sub-sentence streaming (server):** `synthesize_sentence` → `synthesize_sentence_streaming` — drives the crate's `generate_with_voice_streaming` with a `tokio::mpsc` channel, forwarding each decoded PCM chunk (~every 4 frames ≈ 320 ms) to the client as its own binary frame from inside `spawn_blocking`. Added a chunk-count + first-chunk-latency to the RTF log to prove streaming. Added `[profile.dev.package."*"] opt-level = 3` so dev builds aren't the RTF-8 trap.
- **Transcript sync:** new `tts_sentence` protocol message (the spoken/stripped text) sent before each sentence's chunks. Time-to-first-audio dropped from ~4 s (whole sentence) to a few hundred ms.
- **Web client (`astra-web`, React/TS):** rewrote `audio/playback.ts` as a Web Audio **scheduling cursor** (`startTtsStream`/`enqueueChunk`/`endTtsStream`) — chunks play gapless, back-to-back, as they arrive; `context/ws.tsx` plays each binary frame on arrival and reveals each sentence's text via a timer set to its scheduled play time; dropped the old buffer-all-then-play-at-`tts_end` path; `useAudio.ts` mirrors playback state from `speakingId`. Confirmed working end-to-end ("works great").
- **Found (deferred):** the mic button does nothing over plain-HTTP LAN — `navigator.mediaDevices.getUserMedia` requires a secure context (HTTPS or `localhost`), and `startRecording`'s silent `catch` hid the failure. Not caused by the TTS work (recording logic unchanged). Fix later via HTTPS / localhost.

---

## 2026-06-25 — TTS → Qwen3-TTS-Rust crate; realtime (RTF 0.80) on the server

- Migrated TTS from `any-tts` (Candle, RTF ~2.5) to the **`cgisky1980/Qwen3-TTS-Rust`** crate — same model family as GGUF via llama.cpp (Vulkan) + ONNX decoder. Git dep pinned to a rev; `ort` pinned `=2.0.0-rc.11`; `features=["vulkan"]`.
- De-risked with an isolated spike first (`tts-spike/`) before touching astra: confirmed it builds, runs on the server's RTX 5060 Ti via Vulkan, and clears RTF < 1.
- Rewrote the synthesis layer: `AppState.tts` `Arc<dyn TtsModel>` → `Arc<Mutex<TtsEngine>>` (`generate_with_voice` takes `&mut self`); `load_tts_model` is async (awaited directly — the crate's `new` is genuinely async, no nested-runtime trap); `synthesize_sentence` runs in `spawn_blocking` holding the std mutex; speaker is `Arc<VoiceFile>` loaded once. `TTS_SAMPLE_RATE` const (24 kHz) replaces the model accessor.
- Config: `TTS_QUANT` (none|q5_k_m|q8_0) replaces `TTS_DTYPE`/`TTS_MODEL_ID`; `TTS_MAX_TOKENS` → `set_max_steps`. Native libs dlopen'd from `./runtime` (CWD); models in `.astra/models/tts/`, speakers in `.astra/models/tts/speakers/`.
- **Verified on the server: RTF 0.80** (5.0 s audio in 4.0 s) with whisper (CUDA) + the resident Ollama 9B (CUDA) sharing the GPU — the three native stacks coexist in one process cleanly.
- **Hard-won lesson:** a debug build runs at RTF ~8; `--release` drops it to ~0.8. The per-frame hot path is the crate's *own Rust* (projection, sampling, ONNX marshalling), which debug-mode pessimizes ~10× and starves the GPU. Added `[profile.dev.package."*"] opt-level = 3`. The mid-debug CUDA/Vulkan-contention theory was wrong — coexistence is fine.
- Findings: quantization barely moves RTF here (talker isn't memory-bound) but Q5 saves ~1.7 GB VRAM; the crate's GPU ONNX path is DirectML/Windows-only (Linux decode is CPU, but not the bottleneck); the crate exposes a streaming API (`generate_with_voice_streaming`) — next up for time-to-first-audio.

---

## 2026-06-24 — TTS migrated to any-tts/Qwen3-TTS; latency tuning

- Diagnosed the kokoro-tiny gappy-audio root cause: its `load_voices` hardcodes `voicepack[0]` instead of `voice[len(tokens)]` — wrong style vector → bad durations. Verified against reference `thewh1teagle/kokoro-onnx`.
- Migrated TTS from `kokoro-tiny` to **`any-tts v0.1.2`** with `ModelType::Qwen3Tts` (Qwen3-TTS-1.7B-CustomVoice). `AppState.tts`: `Arc<Mutex<TtsEngine>>` → `Arc<dyn TtsModel>` (synthesize takes `&self`). Load wrapped in `spawn_blocking` (any-tts's nested runtime). Cargo: `kokoro-tiny` → `any-tts` (`default-features=false`, `qwen3-tts`/`download`/`cuda`); added `tracing` + `tracing-subscriber`.
- Renamed `KOKORO_VOICE` → `TTS_VOICE`; added runtime knobs `TTS_MAX_TOKENS`, `TTS_MODEL_ID`, `TTS_DTYPE` (tune model/dtype/length via `astra.conf` + restart, no slow candle rebuild).
- Added `TtsStart(TtsStartPayload)` to the protocol — audio format sent before the first chunk so the client can configure playback.
- Built per-sentence streaming → inline interleave (synthesize each sentence as it streams), then **temporarily reverted to whole-clip** synthesis to isolate latency measurements (`take_sentences`/`speak_sentence` kept for restore).
- Added observability: `tracing_subscriber` in `main.rs`; model-load timing, per-synth **RTF** logging, LLM first-token/streamed timing.
- **Key finding:** Qwen3-TTS-1.7B runs at **RTF ~2.4–2.8** on the dev RTX 3070 — too slow for realtime streaming (synthesis can't keep pace with playback). EOS fires correctly (not over-generation); LLM is fast (~1.4 s first token).
- Investigated the "Candle fp32 fallback" hypothesis — **disproven**: any-tts runs BF16 on CUDA. OOM is tight-8 GB-margin, not 2× bloat.
- Confirmed any-tts has **no streaming synthesis API** (whole-clip only; Qwen3-TTS streaming exists in the model but the binding hardcodes non-streaming).
- Path forward: `TTS_DTYPE=f16` A/B → smaller/faster model → non-autoregressive (Kokoro) if RTF stays > 1; streaming deferred.

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
