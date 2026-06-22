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

Astra is a Rust/Axum WebSocket server that acts as a middleware between WebSocket clients and a local Ollama LLM, with a tool-dispatch layer that lets the model invoke local shell integrations. All client communication is over a single `/ws` WebSocket endpoint.

**Request flow:**

```
WS client → handlers::ws::ws_handler
                ↓
          handle_socket (per-connection loop)
                ↓
          run_agent_loop
                ↓
          backend::ollama::client::chat  (streams from Ollama)
                ↓
          If model returns tool_calls:
            tools::dispatch::dispatch_tool(name, args)
                ↓
            tools::implementations  (tokio::process::Command)
                ↓
          Loop back to Ollama with tool results
                ↓
          Stream text chunks to client as text_chunk frames
```

**Module structure:**

```
src/
├── main.rs                      — server entry point, router
├── backend.rs                   — module root
├── backend/
│   ├── state.rs                 — AppState (tools, system prompt, ollama url, reqwest client)
│   ├── config.rs                — loads .astra/core/ + .astra/user/ markdown into system prompt
│   ├── conversation.rs          — per-connection message history with sliding window
│   ├── protocol.rs              — WebSocket message schema (Envelope, Message enum, payloads)
│   ├── ollama.rs                — module root
│   └── ollama/
│       ├── client.rs            — reqwest wrapper around Ollama /api/chat
│       └── types.rs             — OllamaMessage, ChatRequest, Role enum
├── handlers.rs                  — module root
├── handlers/
│   └── ws.rs                    — WebSocket upgrade handler and agent loop
├── tools.rs                     — module root
└── tools/
    ├── registry.rs              — Tool/ToolFunction structs, register_tools()
    ├── dispatch.rs              — dispatch_tool(name, args) match router
    └── implementations.rs       — async tool implementations (tokio::process::Command)
```

**Adding a new tool:**
1. Add an `async fn` in `src/tools/implementations.rs`.
2. Register a `Tool::new(...)` entry in `src/tools/registry.rs` → `register_tools()`.
3. Add a match arm in `src/tools/dispatch.rs` → `dispatch_tool`.

**Configuration notes:**
- Ollama server URL: read from `OLLAMA_URL` env var at startup (falls back to hardcoded LAN IP in `backend/state.rs`).
- Model name: hardcoded in `handlers/ws.rs` (`"qwen3.5:9b"`).
- System prompt: assembled at startup from `.astra/core/*.md` then `.astra/user/*.md`.
- Server listens on `0.0.0.0:3000`.

## Conventions

- **Rust edition:** 2024. Follow modern Rust 2018+ idioms throughout.
- **Module files:** use the named-file convention — `foo.rs` alongside `foo/` for submodules. Never use `mod.rs` (that is the old Rust 2015 style).
- **Naming:** `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- **Async:** use `tokio::process::Command` (not `std::process::Command`) inside async functions.
- **Error handling:** prefer `?` for propagation; use `expect("reason")` only at startup where failure should be fatal; avoid bare `unwrap()` in production paths.