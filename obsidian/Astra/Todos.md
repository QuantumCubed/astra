---
created: 2026-06-13
updated: 2026-06-13
---

# Todos

[[ASTRA|← Home]]

## In Progress

_Nothing active yet — ready to start Phase 1._

## Backlog

### Phase 1 — WebSocket Foundation

- [ ] Define WebSocket message schema (type discriminator: text_message, text_chunk, tool_call, tool_result, error, audio_chunk)
- [ ] Replace HTTP routes with a `/ws` WebSocket handler in Axum
- [ ] Stateful multi-turn conversation scoped to the WebSocket connection
- [ ] Streaming LLM responses token-by-token over WebSocket

### Phase 2 — Tool Layer

- [ ] Refactor tool implementations into per-tool modules
- [ ] Structured tool error handling
- [ ] Define and implement first real tool

### Later

- [ ] STT/TTS placement decision (see [[Roadmap#Phase 3 — Voice Interface]])

## Done

- [x] Basic Axum server with `/ollama`, `/generate`, `/chat` routes
- [x] Ollama client for `list_models`, `generate`, `chat`
- [x] Tool registry, dispatch, and implementation layer
- [x] Claude Code configured (permissions, cargo check hook, plugins)
- [x] Obsidian vault set up
