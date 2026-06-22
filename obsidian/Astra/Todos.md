---
created: 2026-06-13
updated: 2026-06-21
---

# Todos

[[ASTRA|← Home]]

## In Progress

_Phase 2 — Tool layer improvements._

## Backlog

### Phase 2 — Tool Layer

- [ ] Fix tool implementations to capture stdout (currently return `ExitStatus` only — tool results sent to model are meaningless)
- [ ] Refactor tool implementations into per-tool modules
- [ ] Structured tool error handling (tool failures must not crash the request)
- [ ] Define and implement first real tool

### Later

- [ ] STT/TTS placement decision (see [[Roadmap#Phase 3 — Voice Interface]])

## Done

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
