---
created: 2026-06-13
updated: 2026-06-21
---

# Concepts

[[LEARNING]]

A running log of Rust concepts, patterns, and principles encountered during development of Astra. Entries marked **Claude** were explained during a session. Entries marked **Self-discovered** were figured out independently.

---

## Async Byte Stream Iteration with `StreamExt`
**Date:** 2026-06-21
**Context:** Implementing streaming Ollama responses in `ws_handler.rs`
**Source:** Claude

`reqwest::Response::bytes_stream()` converts an HTTP response body into an async stream of raw byte chunks that arrive as the server sends them. To iterate an async stream, you need the `StreamExt` trait from the `futures` crate — it adds the `.next()` method, which awaits the next chunk. Without importing `StreamExt`, the stream has no way to advance. The pattern `while let Some(Ok(chunk)) = stream.next().await` handles both stream termination (`None`) and chunk errors (`Err`) in one expression.

---

## Line Buffering Pattern with `drain`
**Date:** 2026-06-21
**Context:** Parsing NDJSON from Ollama's streaming response in `ws_handler.rs`
**Source:** Claude

Network byte chunks don't align to application-level boundaries like newlines. A chunk may contain half a JSON line, or two and a half. The line buffering pattern handles this: append each chunk to a `String` buffer, then loop calling `find('\n')` — if a newline is found, a complete line is available. `drain(..=pos)` removes and returns characters from the start of the buffer up to and including the newline, simultaneously extracting the line and leaving any partial next line in place. Without `drain`, you'd need to track offsets manually or clone unnecessarily.

---

## `String::from_utf8_lossy` for Safe Byte Conversion
**Date:** 2026-06-21
**Context:** Converting reqwest byte chunks to strings in `ws_handler.rs`
**Source:** Claude

`String::from_utf8_lossy(&bytes)` converts a byte slice to a string without failing on invalid UTF-8 — any invalid sequences are replaced with the replacement character `\u{FFFD}` (`�`). This is preferable to `String::from_utf8` when the source is network data where perfect encoding cannot be guaranteed. The return type is `Cow<str>` (either borrowed or owned depending on whether replacement was needed), which coerces transparently to `&str` in most contexts.

---

## The `?` Operator for Error Propagation
**Date:** 2026-06-22
**Context:** Implementing `config.rs` to load system prompt files
**Source:** Claude

The `?` operator is shorthand for "return this error to my caller if it failed, otherwise give me the success value." It can only be used in functions that return `Result` or `Option`. When placed after a fallible expression, it unwraps the `Ok` value on success and immediately returns the `Err` to the caller on failure — without needing a match statement. This makes error propagation concise and readable without hiding the fact that failure is possible.

---

## `&mut` References for Mutation Across Function Boundaries
**Date:** 2026-06-22
**Context:** Passing a `String` into `load_dir` for appending in `config.rs`
**Source:** Claude

In Rust, function arguments are immutable by default. To allow a function to modify the caller's data, you pass a mutable reference with `&mut`. The caller passes `&mut value`, and the function declares the parameter as `param: &mut Type`. Only one `&mut` reference to a value can exist at a time — this is enforced by the borrow checker and prevents data races. The function can modify the value through the reference, and the changes are visible to the caller after the function returns.

---

## `Result<(), E>` for Fallible Functions With No Return Value
**Date:** 2026-06-22
**Context:** Return type of `load_dir` in `config.rs`
**Source:** Claude

When a function can fail but has nothing meaningful to return on success, the return type is `Result<(), E>`. The `()` (unit type) signals "success with no value." The function returns `Ok(())` at the end to indicate success. This is the Rust equivalent of a void function that can error — callers use `?` to propagate the error or `match` to handle it explicitly.

---

## `filter_map`: Combining Filter and Map
**Date:** 2026-06-22
**Context:** Skipping errored directory entries in `config.rs`
**Source:** Claude

`filter_map` is an iterator adapter that applies a closure returning `Option<T>`. Items where the closure returns `None` are dropped; items where it returns `Some(value)` are kept and unwrapped. It is equivalent to chaining `.map()` and `.filter()` but more concise. A common use is `.filter_map(|x| x.ok())` on an iterator of `Result` values — it silently discards errors and keeps only the successes.

---

## `Option::map_or`: Default Value for `None`
**Date:** 2026-06-22
**Context:** Filtering `.md` files by extension in `config.rs`
**Source:** Claude

`map_or(default, |val| expression)` on an `Option` returns `default` when the option is `None`, and applies the closure to the inner value when it is `Some`. It is a concise alternative to a `match` when you need one outcome for `None` and a transformation for `Some`. In `path.extension().map_or(false, |ext| ext == "md")`, the `false` default handles paths with no extension cleanly without panicking.

---

## Iterator Chains: `.iter().map().sum()`
**Date:** 2026-06-21
**Context:** Writing the token estimator in `conversation.rs`
**Source:** Claude

Iterators in Rust are lazy — they don't do anything until consumed. `.iter()` gives you an iterator over references to each element. `.map(|item| expression)` transforms each element using a closure (an inline anonymous function written as `|parameter| body`). `.sum()` consumes the iterator and adds all the values together. The closure parameter name is arbitrary — it's just what you're calling each element inside the transformation. Rust automatically dereferences `&T` when accessing fields, so you can write `msg.content` even when `msg` is `&Message`.

---

## Serde Adjacently Tagged Enums
**Date:** 2026-06-21
**Context:** Designing the WebSocket message schema in `src/protocol.rs`
**Source:** Claude

Serde supports several ways to serialize enums. "Adjacently tagged" produces a JSON object with a separate key for the variant name and another for its contents: `{ "type": "text_message", "payload": { ... } }`. You get this with two attributes on the enum: `#[serde(tag = "type", content = "payload")]`. The `tag` attribute names the discriminator field and `content` names the wrapper field. Adding `rename_all = "snake_case"` automatically converts Rust variant names like `TextMessage` to `"text_message"` in JSON.

---

## `#[serde(flatten)]`
**Date:** 2026-06-21
**Context:** Adding `request_id` to the WebSocket message envelope alongside the tagged enum fields
**Source:** Claude

By default, serde nests a struct field as a sub-object. `#[serde(flatten)]` inlines the fields of the annotated field into the parent object instead. Used on a struct field that holds an enum or another struct, it merges their serialized keys into the same JSON level. This lets you combine a tagged enum (which handles `type` and `payload`) with extra fields like `request_id` in the same flat JSON object without nesting.

---

## `Vec<T>` Derefs to `&[T]`, `VecDeque<T>` Does Not
**Date:** 2026-06-21
**Context:** Deciding between `Vec` and `VecDeque` for conversation history storage
**Source:** Claude

`Vec<T>` implements `Deref<Target = [T]>`, meaning a `&Vec<T>` automatically coerces to `&[T]` (a slice). This is why you can return `&self.history` from a method declared to return `&[T]`. `VecDeque<T>` does not implement this because its internal storage may be split across two regions and isn't guaranteed to be contiguous. To get a slice from a `VecDeque` you'd have to call `.make_contiguous()` which requires `&mut self`, changing the method signature in an undesirable way.

---

## Axum WebSocket Upgrade Pattern
**Date:** 2026-06-21
**Context:** Setting up the `/ws` endpoint in `main.rs` and `ws_handler.rs`
**Source:** Claude

WebSocket handlers in Axum work differently from regular HTTP handlers. The route receives a `WebSocketUpgrade` extractor instead of a request body. You call `.on_upgrade(|socket| async { ... })` on it, which performs the HTTP→WebSocket handshake and hands you the actual socket inside an async closure. From that point the handler is a long-running async task that owns the socket for the lifetime of the connection — it's not a request/response cycle.

---

## Unwrapping `Option<Result<T, E>>` with `while let`
**Date:** 2026-06-21
**Context:** Fixing the WebSocket receive loop in `ws_handler.rs`
**Source:** Claude

`socket.recv()` returns `Option<Result<T, E>>` — two layers of wrapping. `Option` signals whether the stream is still open; `Result` signals whether the message was valid. You can unwrap both in a single `while let` pattern: `while let Some(Ok(msg)) = socket.recv().await`. This advances the loop only when both layers succeed, and exits cleanly when the socket closes (`None`) or produces an error (`Err`). The alternative is to unwrap them separately, but the combined form is idiomatic for this pattern.

---
