---
created: 2026-06-13
updated: 2026-06-20
---

# Todos

[[ASTRA|← Home]]

## In Progress

_Nothing active yet — ready to start Phase 1._

## Backlog

### Phase 1 — WebSocket Foundation

**Step 1 — Basic text flow**
- [ ] Define WebSocket message schema (type discriminator: text_message, text_chunk, tool_call, tool_result, error, audio_chunk)
- [ ] Replace HTTP routes with a `/ws` WebSocket handler in Axum
- [ ] Stateful multi-turn conversation scoped to the WebSocket connection
- [ ] Streaming LLM responses token-by-token; branch on `content` vs `tool_calls` in first chunk

**Step 2 — System configuration**
- [ ] Create `config/core/` and `config/user/` directory structure with starter files
- [ ] Load and assemble config files into a single system prompt at startup
- [ ] Compute sliding window size N dynamically from context window minus system prompt size
- [ ] Enforce sliding window on messages array before every Ollama request

### Phase 2 — Tool Layer

- [ ] Refactor tool implementations into per-tool modules
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
