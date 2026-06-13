---
created: 2026-06-13
updated: 2026-06-13
---

# Decisions

[[ASTRA|← Home]]

A log of significant technical and architectural decisions, including the reasoning behind them.

---

## 2026-06-13 — Ollama URL and model name are hardcoded

**Decision:** `BASE_URL` in `ollama_client/async_client.rs` and the model name in `req_handler.rs` are intentionally hardcoded constants, not config values.

**Reasoning:** The project is in early development. Extracting these to environment variables or a config file adds complexity before there is a real need. When multi-environment support or model switching becomes a requirement, this will be revisited.

---

## 2026-06-13 — Tool implementations use shell commands via `std::process::Command`

**Decision:** Tools invoke shell scripts or system commands directly rather than using Rust libraries.

**Reasoning:** Keeps the integration layer simple and language-agnostic. Shell scripts are easy to write and test independently. If a tool grows complex enough to warrant a native Rust implementation, it can be migrated at that point.

---

## 2026-06-13 — WebSocket as the primary transport layer

**Decision:** All client-server communication will be built on WebSocket from the start, including the text interface.

**Reasoning:** Voice is a core target feature. Migrating from HTTP/SSE to WebSocket once a web client is already built against it is a costly refactor touching both ends. Starting with WebSocket for text means voice can be added as a new message type over the same connection without changing the transport layer. The upfront complexity cost is lower than the future refactor cost.

---

## 2026-06-13 — Axum with shared `AppState` for tool registry

**Decision:** Tools are registered at startup into `AppState`, which is cloned into every request via Axum's `State` extractor.

**Reasoning:** Tools don't change at runtime, so a shared immutable list owned by `AppState` is the right fit. Avoids global state and keeps everything within Axum's ownership model.
