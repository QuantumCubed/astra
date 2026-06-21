---
created: 2026-06-13
updated: 2026-06-21
---

# Progress

[[ASTRA|← Home]]

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
