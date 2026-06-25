---
created: 2026-06-25
updated: 2026-06-25
scope: audio/TTS refactor (handlers/ws.rs, backend/audio/, state.rs, config.rs, protocol.rs, main.rs) + tools/STT security scan
---

# Code Review — audio/TTS refactor — 2026-06-25

[[REVIEWS|← Reviews]]

## Summary

Reviewed the code touched by the Qwen3-TTS-Rust migration and the streaming/transcript-sync work, plus a security pass over `stt.rs` and `tools/implementations.rs`. Overall the refactored pipeline is in good shape: no `unwrap()` in production paths, no `unsafe`, no command injection (all `Command` args are literals), and the blocking FFI (TTS synthesis, whisper) is correctly offloaded with `spawn_blocking`. The startup `expect`/`panic!` calls in `main.rs` and `state.rs` are consistent with the project convention (fatal-at-startup) and are **not** flagged. **7 findings: 0 critical, 2 warnings, 5 suggestions.** The most important is a blocking `std::fs::write` left in an async function; the next is that WebSocket send errors are discarded, so a client disconnecting mid-reply doesn't stop synthesis.

## Findings

### Critical

None.

### Warnings

**1 — Blocking `std::fs::write` inside an async fn**
- **File**: `src/handlers/ws.rs` line 359
- **Issue**: `std::fs::write("debug_audio.raw", &dump)` is a synchronous filesystem call, but it runs inside `run_agent_loop`, which is `async`.
- **Why**: Tokio multiplexes many async tasks onto a small pool of worker threads. A synchronous I/O call doesn't yield — it parks the entire worker thread until the OS finishes writing (here, potentially hundreds of KB of PCM every turn). While that thread is parked it can't make progress on any other connection. This is the same hazard the project already avoids by mandating `tokio::process::Command` over `std::process::Command`.
- **Direction**: use `tokio::fs::write(...).await`, or wrap it in `spawn_blocking` like the TTS/whisper calls. Since this is a debug artifact, also consider gating it behind a flag (or removing it) so it isn't written on every production turn.

**2 — WebSocket send errors are silently discarded**
- **File**: `src/handlers/ws.rs` lines 59, 96, 121, 154, 167, 187, 286, 370 (the `let _ = socket.send(...).await;` pattern)
- **Issue**: Every `socket.send(...)` result is dropped with `let _ =`. If the client has disconnected, the error is ignored and the loop continues.
- **Why**: In a voice turn this has real cost. If the user closes the tab mid-reply, the server keeps synthesizing every remaining sentence (the expensive GPU work) and keeps pushing frames into a dead socket — all wasted. Discarding the `Result` also throws away the one signal that the connection broke, so nothing can react to it. Dropping a `Result` like this is precisely what Rust's `#[must_use]` on `Result` is trying to warn you about.
- **Direction**: inside the long loops (the per-chunk streaming loop in `speak_sentence` and the agent loop), check the result and stop the turn when a send fails — e.g. `if socket.send(...).await.is_err() { return; }`. That aborts synthesis the moment the client is gone. Fire-and-forget `let _` is defensible for a single best-effort frame (like an error reply), but not in a loop that does heavy work per iteration.

### Suggestions

**3 — `ws_handler` is `async` but never `.await`s**
- **File**: `src/handlers/ws.rs` line 17
- **Issue**: `ws_handler` is declared `async`, but its body (`ws.on_upgrade(...)`) returns synchronously; there is no `.await`.
- **Why**: An `async fn` with no `.await` produces a future that completes immediately — the `async` keyword changes the return type but buys nothing here. It's harmless (Axum accepts sync and async handlers), but it can mislead a reader into thinking the function suspends. Worth internalizing as you learn: `async` isn't free decoration, it's a type change.
- **Direction**: it could be a plain `fn ws_handler(...) -> Response`. Many Axum examples leave such handlers `async` by convention, so this is a judgment call — but a useful distinction to understand.

**4 — `.astra/astra.conf` is read and re-parsed once per key**
- **File**: `src/backend/config.rs` (`load_ollama_url`, `load_whisper_model_path`, `load_tts_voice`, `load_tts_max_tokens`, `load_tts_quant`)
- **Issue**: Each loader independently calls `std::fs::read_to_string(".astra/astra.conf")` and scans the lines for its key. The refactor added `load_tts_quant`, a fifth pass over the same file.
- **Why**: It's repetitive (the same read-and-scan boilerplate copied five times) and re-reads the file from disk ~5× at startup. Performance is irrelevant at startup, but the duplication is the real smell — five near-identical parsers invite drift as the config format evolves.
- **Direction**: parse the file once into a `HashMap<String, String>` (or a small `Config` struct) at startup and have the `load_*` accessors read from that — one read, one parser, one place to change the format.

**5 — `run_agent_loop` carries several distinct responsibilities**
- **File**: `src/handlers/ws.rs` lines 211–373
- **Issue**: One ~160-line function drives the Ollama request loop, parses the NDJSON stream, forwards text chunks, detects and dispatches tool calls, interleaves per-sentence TTS, speaks the trailing sentence, writes the debug dump, and sends `TtsEnd`.
- **Why**: Long multi-purpose functions are harder to follow and change safely. A tweak to the speech lifecycle means scrolling past the LLM-streaming logic, and the two concerns share mutable state (`unspoken`, `tts_started`, `all_samples`), which makes subtle ordering bugs easy to introduce. It's not wrong — it just gets riskier each time it's touched.
- **Direction**: consider extracting the TTS side (the `take_sentences`/`speak_sentence` interleave plus the tail, `TtsEnd`, and dump) into its own helper, so the LLM loop and the speech lifecycle are separable. Flagging only — no need to refactor now.

**6 — Hardcoded Ollama model name**
- **File**: `src/handlers/ws.rs` line 227
- **Issue**: `model: "qwen3.5:9b".to_string()` is a literal.
- **Why**: This is documented as intentional, so it isn't a bug. But now that the other runtime knobs (`OLLAMA_ENDPOINT`, `WHISPER_MODEL`, `TTS_*`) live in `astra.conf`, the model name is the odd one out — switching models still requires a recompile, unlike everything else.
- **Direction**: when convenient, read it from `astra.conf` (e.g. an `OLLAMA_MODEL` key with a sensible default), consistent with the other runtime settings.

**7 — `AppState` deep-clones `tools` and `system_prompt` per connection**
- **File**: `src/backend/state.rs` (`#[derive(Clone)] AppState`; fields `tools: Vec<Tool>`, `system_prompt: String`)
- **Issue**: Axum clones `AppState` into every connection. The `Arc<…>` fields clone cheaply, but `tools` (a `Vec<Tool>`) and `system_prompt` (a `String`, possibly several KB) are deep-copied each time.
- **Why**: It's correct, just wasteful — both are read-only after startup, so duplicating them per connection is avoidable. It mirrors a sharing decision already made for `whisper_ctx`/`tts` (which use `Arc`). On a home server with few clients it's immaterial, but it's an easy, idiomatic win. (Pre-existing — not introduced by the refactor.)
- **Direction**: wrap the read-only owned fields in `Arc` (e.g. `Arc<Vec<Tool>>` / `Arc<String>`, or `Arc<[Tool]>` / `Arc<str>`) so cloning `AppState` becomes all pointer bumps.

## Discussion

**2026-06-25 — all findings addressed** (refactored on request):
- **W1**: the `debug_audio.raw` write is commented out in `TtsStream::finish`, kept as ready-to-use scaffolding; the commented form uses `tokio::fs` so re-enabling won't block the runtime.
- **W2**: sends now go through a `send_frame` helper returning `false` on failure; `run_agent_loop` and `TtsStream` stop the turn when the client disconnects, so synthesis no longer runs for a gone client.
- **S3**: `ws_handler` is a plain `fn`.
- **S4**: `astra.conf` is parsed once via `load_conf()` into a `HashMap`; thin accessors read from it.
- **S5**: the speech lifecycle is extracted into a `TtsStream` struct (`feed`/`speak`/`finish`), separate from the LLM loop.
- **S6**: the model name is now an `OLLAMA_MODEL` config key (default `qwen3.5:9b`).
- **S7**: `AppState.tools`/`system_prompt` are `Arc<Vec<Tool>>`/`Arc<str>` (and `ChatRequest.tools: Arc<Vec<Tool>>`), so cloning `AppState` per connection is pointer bumps.

