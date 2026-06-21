---
created: 2026-06-13
updated: 2026-06-21
---

# Architecture

[[ASTRA|← Home]]

## Overview

Astra is a Rust/Axum WebSocket server running on port `3000`. It bridges WebSocket clients to a local Ollama instance, maintaining per-connection conversation state and dispatching tool calls returned by the model.

## Request Flow

```
WebSocket client
    └── Axum router (src/main.rs)
            └── GET /ws → WebSocket upgrade
                    ↓
            ws_handler::handle_socket (per-connection async loop)
                    ├── Parses incoming Envelope (src/protocol.rs)
                    ├── Maintains conversation via Conversation (src/conversation.rs)
                    │       └── Sliding window token management
                    └── ollama_client::async_client (HTTP to Ollama REST API)
                            ↓
                    model response:
                      tool_calls? → dispatch → implementations → continue loop
                      text?       → stream TextChunk envelopes back to client
```

## Modules

| Module | Responsibility |
|---|---|
| `src/main.rs` | Router setup, server startup, AppState init |
| `src/state.rs` | `AppState` — shared state holding registered tools |
| `src/protocol.rs` | WebSocket message schema — `Envelope`, `Message` enum, payload structs |
| `src/handlers/ws_handler.rs` | WebSocket connection handler, per-connection message loop |
| `src/conversation.rs` | Conversation state — message history, sliding window enforcement |
| `src/ollama_client/async_client.rs` | HTTP client wrapper for Ollama API |
| `src/handlers/req_handler.rs` | Legacy HTTP handlers (to be retired); owns `OllamaMessage`, `ChatRequest` types until refactored |
| `src/tools/registry.rs` | Tool definitions, `register_tools()` |
| `src/tools/dispatch.rs` | Routes tool call by name to implementation |
| `src/tools/implementations.rs` | Concrete tool logic (shell commands) |
| `src/integrations/script/` | Shell scripts invoked by tool implementations |

## Conversation State

Ollama is fully stateless — it has no memory between API calls. Astra owns the `messages` array for each active session and replays the full history on every request to Ollama. The system prompt is prepended to this array on every call.

To prevent unbounded growth, Astra applies a **sliding window**: always keep the system prompt + the last N message turns, dropping the oldest when the limit is reached. N is computed at startup as `max_context_tokens - system_prompt_tokens - response_buffer` — the system prompt is always guaranteed to fit; conversation history fills the remaining space.

Current storage: **in-memory per session** (lost on server restart). Persistence deferred.

## Model Configuration

Behavioral config is split into two layers, assembled at startup into a single system prompt injected into every Ollama request:

```
config/
  core/
    identity.md       # what Astra is, what it can do
    tools.md          # rules around tool use
    behavior.md       # hard constraints
  user/
    personality.md    # name, tone, persona
    collaboration.md  # how it works with the user
    coding-style.md   # preferences when writing code
```

Core is loaded first, user config appended after. Core uses hard constraint language; user config covers softer preferences. User files can be absent without error. The combined token count determines the sliding window size N.

Hardware/runtime tuning (`num_ctx`, GPU offload, thread count) is managed via an Ollama Modelfile on the host machine and is outside Astra's scope.

## Current Tools

| Tool | Description |
|---|---|
| `echo_hello_world` | Runs `echo.sh` — prints "Hello, World!" |
| `list_contents` | Runs `ls` in the working directory |
