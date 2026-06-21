---
created: 2026-06-13
updated: 2026-06-20
---

# Architecture

[[ASTRA|← Home]]

## Overview

Astra is a Rust/Axum HTTP server running on port `3000`. It sits between HTTP clients and a remote Ollama instance, forwarding requests and handling tool calls that the LLM returns.

## Request Flow

```
HTTP client
    └── Axum router (src/main.rs)
            ├── GET  /ollama    → list available models
            ├── POST /generate  → single-turn text generation
            └── POST /chat      → chat with tool-use loop
                    ↓
            ollama_client::async_client
            (HTTP to Ollama REST API)
                    ↓
            model response contains tool_calls?
              YES → tools::dispatch::dispatch_tool()
                        └── tools::implementations
                                └── shell commands / scripts
              NO  → return message directly
```

## Modules

| Module | Responsibility |
|---|---|
| `src/main.rs` | Router setup, server startup, AppState init |
| `src/state.rs` | `AppState` — shared state holding registered tools |
| `src/handlers/req_handler.rs` | Axum handlers, request/response types |
| `src/ollama_client/async_client.rs` | HTTP client wrapper for Ollama API |
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
