---
created: 2026-06-13
updated: 2026-06-13
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

## Current Tools

| Tool | Description |
|---|---|
| `echo_hello_world` | Runs `echo.sh` — prints "Hello, World!" |
| `list_contents` | Runs `ls` in the working directory |
