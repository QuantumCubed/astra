---
created: 2026-06-13
updated: 2026-06-13
---

# Roadmap

[[ASTRA|← Home]]

A phased plan for Astra's development. Phases are sequential — each one builds the foundation the next depends on.

---

## Phase 1 — WebSocket Foundation

**Goal:** Replace the current HTTP quickstart with a WebSocket server and a well-defined message protocol. This is the transport layer everything else runs on.

**Why first:** Voice requires WebSocket. Building the text interface on WebSocket now means voice is a new message type, not a migration.

### Tasks

- [ ] Define the WebSocket message schema (see [[Roadmap#Message Protocol|Message Protocol]] below)
- [ ] Replace Axum HTTP routes with a WebSocket handler (`/ws` endpoint)
- [ ] Implement text chat over WebSocket (stateful — connection holds conversation history)
- [ ] Multi-turn conversation: keep message history on the server, scoped to the WebSocket connection
- [ ] Streaming responses: stream LLM output token-by-token rather than waiting for full response

### Message Protocol

All messages are JSON with a `type` discriminator. Initial types:

| Type | Direction | Payload |
|---|---|---|
| `text_message` | Client → Server | `{ "content": "..." }` |
| `text_chunk` | Server → Client | `{ "content": "...", "done": bool }` |
| `tool_call` | Server → Client | `{ "name": "...", "args": {...} }` |
| `tool_result` | Server → Client | `{ "name": "...", "result": "..." }` |
| `error` | Server → Client | `{ "message": "...", "code": "..." }` |
| `audio_chunk` | Both | `{ "data": "<base64>", "done": bool }` — reserved for Phase 3 |

The `audio_chunk` type is reserved but not implemented until Phase 3 — defining it now avoids breaking changes to the protocol later.

---

## Phase 2 — Tool Layer

**Goal:** Build tools that are actually useful for a home AI assistant.

**Why after Phase 1:** Tools only matter once there is a real conversational interface to call them through.

### Tasks

- [ ] Design the tool interface properly — each tool should be its own module, not a single `implementations.rs`
- [ ] Decide how tools are registered — static list vs. dynamic loading
- [ ] Implement first real tool (TBD — candidate: file read, web search, home automation command)
- [ ] Structured tool error handling — tool failures should not crash the request
- [ ] Tool result streaming — long-running tools should stream progress

---

## Phase 3 — Voice Interface

**Goal:** Real-time voice I/O over the existing WebSocket connection.

**Why after Phase 2:** Voice makes no sense without a solid conversational loop underneath it. Phase 1 + 2 provide that.

### Key Design Decision (pending)

**Where does STT/TTS run?** This is the biggest latency decision in the entire project.

| Option | Latency | Complexity |
|---|---|---|
| Client-side STT → send text | Low (no audio upload) | Client must do STT |
| Server-side STT (Whisper) | Medium (audio upload) | Server owns full pipeline |
| Client-side STT + server-side TTS | Mixed | Hybrid |

A decision here should be logged to [[Decisions]] when made.

### Tasks

- [ ] Decide STT/TTS placement (see above)
- [ ] Implement `audio_chunk` message handling in the WebSocket layer
- [ ] Integrate STT model (Whisper or client-side alternative)
- [ ] Integrate TTS model
- [ ] Voice activity detection — know when the user has finished speaking
- [ ] Interruption handling — user speaks while assistant is responding

---

## Phase 4 — Web Client

**Goal:** A browser-based interface for text and voice interaction.

**Why after Phase 3:** Voice in the browser is the most constrained environment (Web Audio API, microphone permissions, codec support). Building the server-side voice pipeline first means the client can be designed around known constraints.

### Tasks

- [ ] Choose a framework (or no framework — depends on complexity)
- [ ] Connect to the WebSocket server from the browser
- [ ] Text chat UI
- [ ] Voice UI — push-to-talk or VAD
- [ ] Streaming response rendering

---

## Phase 5 — Agents & Expansion

**Goal:** Extend Astra with agentic capabilities and additional clients.

This phase is intentionally loose — the right shape will be clearer after Phases 1–4.

### Candidates

- [ ] Multi-step agent loop (plan → act → observe → repeat)
- [ ] Smart home integration
- [ ] Desktop client
- [ ] Mobile client
- [ ] Memory / user profile persistence

---

## What Is Not In Scope Yet

- Authentication / multi-user support
- Cloud deployment (this is a home server)
- Model fine-tuning
