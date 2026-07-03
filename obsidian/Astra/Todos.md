---
created: 2026-06-13
updated: 2026-07-02
---

# Todos

[[ASTRA|← Home]]

## In Progress

- [ ] Full mic → STT → LLM → TTS round-trip test with the web client (blocked: mic capture needs a secure context — see Backlog)

## Backlog

### Multi-User Accounts & Auth (planned — see [[Decisions]] 2026-07-02, [[Roadmap]] Phase 3)

- [ ] Write `migrations/0001_create_users.sql` .. `0004_create_messages.sql`
- [ ] `backend/db.rs` + `backend/db/{users,sessions,conversations,messages}.rs`
- [ ] `backend/auth.rs` + `backend/auth/{password,session}.rs`
- [ ] CLI `astra user create <username>` subcommand with `rpassword` hidden input
- [ ] New protocol messages in `backend/protocol.rs` (`Login`, `ResumeSession`, `AuthResult`, `ListConversations`, `CreateConversation`, `SwitchConversation`, `ConversationSwitched`)
- [ ] `ws.rs`: `authenticate()` pre-auth gate with 15s `tokio::time::timeout`
- [ ] `conversation.rs`: `Conversation::load`/persist-per-turn rework, DB is source of truth, in-memory vec stays a context-window trim only
- [ ] `SwitchConversation` ownership check (`WHERE id = ? AND user_id = ?`) — do not skip
- [ ] `AppState.db: SqlitePool` field; `main.rs` connect+migrate at startup, ahead of `AppState::new()`
- [ ] Unit tests: `auth::password`, `auth::session`, protocol round-trip, `db::*` ownership isolation
- [ ] Manual end-to-end verification via `websocat` (login, resume, conversation switch, negative tests)
- [ ] Update `Architecture.md` once the above lands

### Phase 2 — Voice Pipeline

- [ ] Microphone capture fails over plain-HTTP LAN — `getUserMedia` needs a secure context (HTTPS or `localhost`); serve the web client over HTTPS (mkcert/tunnel) for LAN mic access
- [ ] Surface mic-capture errors in the web client — `startRecording`'s silent `catch` hides secure-context / permission / no-device failures (the button just looks dead)
- [ ] (Minor) Web client: guard the overlapping-response case where a new reply's `speakingId` can be cleared by the previous reply's last-chunk `onended`
- [ ] Interruption handling — user speaks while the assistant is responding
- [ ] (Deferred, only if needed) GPU ONNX decode — would need a crate fork to add a CUDA execution provider; unnecessary while RTF < 1

### Home Assistant

- [ ] Area-based device toggling — toggle all devices in a room by area name (e.g. "turn off the bonus room lights")
- [ ] Implement event subscriptions in `event_loop` (currently stubs only)
- [ ] `call_service` with `service_data` for services that require parameters (e.g. light brightness, colour)
- [ ] Refactor Spotify into `SpotifyCtx` — move `spotify_token`, `spotify_devices` out of raw `AppState` fields; mirrors the `HaClient` modular pattern

### Phase 3 — Tool Layer

- [ ] Refactor tool implementations into per-tool modules
- [ ] Structured tool error handling (tool failures must not crash the request)
- [ ] Spotify: implement 401 → refresh token → retry path in play/pause/search implementations
- [ ] Spotify: bootstrapping HTTP endpoint to automate OAuth flow and write tokens to `astra.conf`
- [ ] Spotify: refactor into `SpotifyCtx` struct — move `spotify_token`, `spotify_devices` out of raw `AppState` fields; add `integrations/spotify/config.rs`
- [ ] Spotify: implement `spotify_swap_playback(from_device, to_device)` — seamlessly transfer active playback from one device to another
- [ ] Spotify: implement `spotify_queue(uri)` — add a track to the current play queue

## Done

- [x] Home Assistant: `ha_get_devices`, `ha_toggle_device`, `ha_reconnect` tools wired end-to-end through `registry.rs`, `implementations.rs`, `dispatch.rs`
- [x] Home Assistant: `HaClient::reconnect()` + `establish()` helper — reconnects the WebSocket without restarting the server; old `event_loop` dies naturally when the dropped connection closes
- [x] Home Assistant: `call_service(domain, service, entity_id)` on `HaClient` via `send_command` abstraction
- [x] Home Assistant: `get_devices()` — three-way join of `get_states` + `get_entity_registry` + `get_area_registry` via `tokio::join!`; filtered to `disabled_by == null` + `should_expose == true`; returns `Vec<HaDevice>`
- [x] Home Assistant: `HaDevice` struct in `ha/types.rs` (`entity_id`, `friendly_name`, `aliases`, `area`, `state`)
- [x] Home Assistant: `send_command` private method on `HaClient` — abstracts id injection, pending-map registration, sink lock, and success check; all public methods are now one-liners
- [x] Home Assistant: `get_entity_registry` and `get_area_registry` methods on `HaClient`
- [x] Home Assistant: `HaClient` struct, `connect()` auth handshake, split WebSocket, background `event_loop` (stub), pending-map for response matching; `Option<Arc<HaClient>>` in `AppState`; graceful startup on HA unavailable
- [x] Spotify: `spotify_search(query)` tool — searches tracks, albums, playlists; returns `Vec<(name, uri)>` as JSON for the model to choose from; null items handled via `Vec<Option<T>>` + `.flatten()`
- [x] Spotify: `spotify_play_content(uri, device_name?)` tool — plays a URI on a named or active device; routes `spotify:track:` to `uris[]`, others to `context_uri`; falls back to active device if name not in cache
- [x] Spotify: `spotify_pause_content` tool — pauses on active device
- [x] Spotify: `spotify_resume_content` tool — resumes on active device
- [x] reqwest `"query"` feature added to `Cargo.toml` (`.query()` is not in core in 0.13)
- [x] Spotify: `spotify_get_devices` tool — reads cached `state.spotify_devices`, returns device names as JSON; `dispatch_tool` updated to accept `&AppState`
- [x] Spotify: `integrations/spotify/spotify_connection.rs` — `refresh_access_token` (form POST, Basic auth) and `get_devices` (Bearer auth, returns `HashMap<name, id>`)
- [x] Spotify: `AppState` extended with `spotify_token: Arc<Mutex<String>>` and `spotify_devices: Arc<Mutex<HashMap<String,String>>>`; access token minted at startup
- [x] Spotify: credentials (`SPOTIFY_CLIENT_ID`, `SPOTIFY_CLIENT_SECRET`, `SPOTIFY_REFRESH_TOKEN`) added to `astra.conf` and reader functions added to `config.rs`
- [x] `reqwest` `form` feature added to `Cargo.toml`
- [x] Single shared `reqwest::Client` for all integrations — moved from inline `Self {}` to pre-constructed local in `AppState::new()`

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
