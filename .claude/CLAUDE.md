# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Vision

Astra is a home AI server — a local, low-latency AI assistant with voice, web, and eventually desktop/mobile clients. It runs on a home server and talks to a local Ollama instance; there is no cloud dependency by design.

**End goals:** real-time voice interface, browser-based text/voice client, smart home integration, agentic task execution.

**Transport commitment:** all client-server communication is WebSocket. This is a firm architectural decision — do not suggest HTTP/SSE alternatives. Voice is added as a new message type over the same connection, not a migration.

**Build order:** WebSocket text interface → real tool layer → voice → web client → agents. The current HTTP quickstart code is a placeholder and will be fully replaced in Phase 1.

See `obsidian/Astra/` for the full project context: `Roadmap.md`, `Decisions.md`, `Architecture.md`, `Todos.md`, and `Progress.md`.

> If the Obsidian notes and the codebase disagree — different module structure, removed routes, changed patterns — flag the discrepancy to the user before proceeding. Either the notes are stale or the code diverged from the plan; the user should decide which is authoritative.

## Commands

```bash
cargo build          # compile
cargo run            # start server on port 3000
cargo test           # run tests
cargo check          # fast type-check without linking
cargo clippy         # lint
```

## Architecture

Astra is a Rust/Axum HTTP server that acts as a middleware between HTTP clients and a remote Ollama LLM, with a tool-dispatch layer that lets the model invoke local shell integrations.

**Request flow:**

```
HTTP client → Axum router (main.rs)
                ├── GET  /ollama    → list models from Ollama
                ├── POST /generate  → single-turn generation
                └── POST /chat      → chat with tool-use loop
                        ↓
              ollama_client::async_client  (talks to Ollama REST API)
                        ↓
              If model returns tool_calls:
                tools::dispatch::dispatch_tool(name, args)
                        ↓
                tools::implementations  (runs shell commands / scripts)
```

**Key modules:**

- `src/state.rs` — `AppState` (shared via Axum `State`): holds the registered `Vec<Tool>`, cloned into every request.
- `src/ollama_client/async_client.rs` — thin reqwest wrapper around Ollama's `/api/tags`, `/api/generate`, and `/api/chat`. The Ollama base URL is hardcoded as `const BASE_URL`.
- `src/handlers/req_handler.rs` — Axum handlers. The `chat_model` handler checks the response for `message.tool_calls` and dispatches each one before returning.
- `src/tools/registry.rs` — `Tool` / `ToolFunction` structs (serialized and sent to Ollama so the model knows what's available). `register_tools()` is the single place to add new tools.
- `src/tools/dispatch.rs` — `dispatch_tool(name, args)` matches on tool name and calls the right implementation. Must be updated in sync with `register_tools()`.
- `src/tools/implementations.rs` — async wrappers around `std::process::Command`. Shell scripts live in `src/integrations/script/`.

**Adding a new tool:**
1. Add a shell script (if needed) under `src/integrations/script/`.
2. Add an `async fn` in `src/tools/implementations.rs`.
3. Register a `Tool::new(...)` entry in `src/tools/registry.rs` → `register_tools()`.
4. Add a match arm in `src/tools/dispatch.rs` → `dispatch_tool`.

**Configuration notes:**
- Ollama server URL: hardcoded in `src/ollama_client/async_client.rs` (`BASE_URL`).
- Model name: hardcoded in `src/handlers/req_handler.rs` (`"qwen3.5:9b"`).
- Server listens on `0.0.0.0:3000`.