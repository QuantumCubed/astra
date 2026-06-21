---
created: 2026-06-13
updated: 2026-06-21
---

# Todos

[[ASTRA|← Home]]

## In Progress

### Phase 1 — WebSocket Foundation

**Step 1 — Basic text flow**
- [x] Define WebSocket message schema (Envelope + Message enum in `src/protocol.rs`)
- [x] Replace HTTP routes with a `/ws` WebSocket handler in Axum (`src/handlers/ws_handler.rs`)
- [ ] Complete `src/conversation.rs` — `add_user_turn`, `add_assistant_turn`, `messages()` 
- [ ] Wire Ollama call into `ws_handler` — parse TextMessage, call chat, send TextChunk response
- [ ] Stateful multi-turn conversation scoped to the WebSocket connection
- [ ] Streaming LLM responses token-by-token; branch on `content` vs `tool_calls` in first chunk

## Backlog

**Step 2 — System configuration**
- [ ] Create `config/core/` and `config/user/` directory structure with starter files
- [ ] Load and assemble config files into a single system prompt at startup
- [ ] Compute sliding window size N dynamically from context window minus system prompt size
- [ ] Enforce sliding window on messages array before every Ollama request

### Phase 2 — Tool Layer

- [ ] Refactor tool implementations into per-tool modules
- [ ] Move `OllamaMessage` and `ChatRequest` types out of `req_handler.rs` into `ollama_client`
- [ ] Structured tool error handling (tool failures must not crash the request)
- [ ] Define and implement first real tool

### Later

- [ ] STT/TTS placement decision (see [[Roadmap#Phase 3 — Voice Interface]])

## Done

- [x] Basic Axum server with `/ollama`, `/generate`, `/chat` routes
- [x] Ollama client for `list_models`, `generate`, `chat`
- [x] Tool registry, dispatch, and implementation layer
- [x] Claude Code configured (permissions, cargo check hook, plugins)
- [x] Obsidian vault set up
