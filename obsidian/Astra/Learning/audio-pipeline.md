---
created: 2026-06-23
updated: 2026-06-23
---

# Audio Pipeline — Implementation Walkthrough

[[LEARNING|← Learning]]

A step-by-step explanation of how `tts.rs`, `stt.rs`, and `ws.rs` were implemented to form the Phase 2 voice pipeline.

---

## `tts.rs`

Two functions: one that loads the model at startup, one that synthesizes audio on demand.

### `load_tts_model(model_path)`

Calls `any_tts::load_model()` with a config that says "Kokoro model, look here for the files." This returns a `Box<dyn TtsModel>` — a heap-allocated value behind a trait object (we don't know or care about the concrete type, only that it implements `TtsModel`). We then wrap it in `Arc::from(...)` to get an `Arc<dyn TtsModel>`.

**Why `Arc` and not just keep the `Box`?** `AppState` is cloned for every WebSocket connection. `Box` can't be cloned. `Arc` can — it just increments a reference count, so all connections share the same underlying model without copying it. The `TtsModel` trait is `Send + Sync`, which means the Arc is safe to share across threads.

**Why `anyhow::Result`?** The any-tts error type is specific to that crate, and the STT layer uses whisper-rs which has its own error type. `anyhow::Error` can swallow any error that implements `std::error::Error`, so using `anyhow::Result` throughout the audio layer gives a consistent, propagatable error type everywhere.

### `synthesize(model, text)`

Kokoro synthesis is synchronous and CPU-intensive. If called directly inside an async function, it would block the thread the async executor is running on, stalling every other WebSocket connection until it finishes.

`spawn_blocking` hands the closure off to a separate thread pool designed for blocking work. The async executor is free while it runs. The closure captures `model` and `text` by move (they're now owned by the thread), calls `model.synthesize(...)`, and returns the raw PCM samples as `Vec<f32>`.

`spawn_blocking` returns a `JoinHandle<anyhow::Result<Vec<f32>>>`. The `.await?` at the end does two things in sequence: `.await` waits for the thread to finish, which gives back a `Result<anyhow::Result<Vec<f32>>, JoinError>` — the outer `Result` handles the case where the thread panicked; the `?` unwraps that, giving us the inner `anyhow::Result<Vec<f32>>`, which becomes the function's return value.

---

## `stt.rs`

Same pattern — load once, transcribe on demand.

### `load_whisper_ctx(model_path)`

Loads the whisper.cpp model file via FFI. The result is a `WhisperContext` which holds the model weights — read-only after this point. We wrap it in `Arc::new(...)` for the same reason as TTS: shared ownership without copying. `whisper-rs` explicitly implements `Send` and `Sync` on `WhisperContext` with `unsafe impl`, meaning the library author guarantees it's safe to share across threads — the weights are read-only, nothing mutates them.

### `transcribe(ctx, audio)`

Again pushed to `spawn_blocking` because whisper.cpp inference is synchronous and slow. Inside the closure:

**1. PCM format conversion.**
The client sends audio as 16kHz mono 16-bit PCM bytes. Each sample is two bytes in little-endian order. We iterate the bytes in pairs with `chunks_exact(2)`, reconstruct each `i16` sample with `i16::from_le_bytes([b[0], b[1]])`, and collect into a `Vec<i16>`.

**2. i16 → f32 conversion.**
Whisper expects samples as `f32` in the range `[-1.0, 1.0]`. `convert_integer_to_float_audio` does the division (divides each i16 by 32768) into a pre-allocated `Vec<f32>`.

**3. Create a `WhisperState`.**
This is the key design decision for thread safety. The `WhisperContext` (shared) holds the weights. Each transcription call creates its own `WhisperState` from it — this is the mutable working memory for that inference run. Multiple connections can transcribe simultaneously without any lock, because they each have their own `WhisperState`.

**4. Configure params.**
`FullParams` carries inference settings — language hint, and several `set_print_*` flags turned off because we don't want whisper.cpp spamming stdout during every request.

**5. Run inference.**
`state.full(params, &samples_f32)` runs the actual transcription. After this, results are available on `state`.

**6. Collect the transcript.**
`state.as_iter()` gives an iterator over `WhisperSegment` values. Each segment is a recognized chunk of speech (a few words or a sentence). We call `.to_str_lossy()` on each one — "lossy" means invalid UTF-8 gets replaced with `?` rather than returning an error — and push it into a `String`. Finally `.trim()` removes leading/trailing whitespace and we return the full transcript.

---

## `ws.rs` — the rewire

### What changed in the `AudioEnd` arm

The old stub was a `_transcript = "stub"` placeholder. The new version:

**1. Drain the audio buffer.**
`std::mem::take(&mut audio_buffer)` moves the buffer contents out and leaves `audio_buffer` as an empty `Vec`. This is the correct way to take ownership of data behind a `&mut` reference without cloning. The audio bytes are now owned and can be passed by value to `transcribe`.

**2. STT.**
Call `transcribe(state.whisper_ctx.clone(), audio).await`. The `.clone()` on the Arc is cheap (just increments a counter), giving `transcribe` its own handle to the shared context.

**3. Send `Transcript` to client.**
If transcription succeeds, serialize and send a `Transcript` frame so the browser can display the recognized text.

**4. Run the agent loop.**
Add the transcript to the conversation as a user turn, then call `run_agent_loop`. The loop was always building `full_content` but discarding it — we changed the return type from `()` to `String` so the voice path can feed the response into TTS. The text path (non-voice messages) simply ignores the return value.

**5. TTS.**
Call `synthesize(state.tts_model.clone(), response).await` with the response text.

**6. Send audio back.**
If synthesis succeeds: convert `Vec<f32>` to raw bytes with `.iter().flat_map(|s| s.to_le_bytes())` — each f32 becomes 4 bytes little-endian. Send the whole thing as a single binary WebSocket frame. Then send a `TtsEnd` JSON frame so the client knows the audio stream is complete.

**7. Error handling.**
Any failure at any step sends an `Error` frame with a code (`STT_ERROR`, `TTS_ERROR`) instead of silently dropping. A `send_error` helper was extracted because the same pattern appeared in both failure branches.

---

## Data flow summary

```
binary WS frames (16kHz mono i16 PCM)
    └── accumulate in audio_buffer
            └── AudioEnd JSON message received
                    └── std::mem::take(audio_buffer) → Vec<u8>
                            └── transcribe() [spawn_blocking]
                                    ├── chunks_exact(2) → Vec<i16>
                                    ├── convert_integer_to_float_audio → Vec<f32>
                                    ├── ctx.create_state() → WhisperState
                                    ├── state.full(params, samples) — inference
                                    └── state.as_iter() → String transcript
                            └── send Transcript frame to client
                            └── run_agent_loop() → String (LLM response)
                            └── synthesize() [spawn_blocking]
                                    └── model.synthesize() → Vec<f32> samples
                            └── samples → f32 LE bytes → binary WS frame
                            └── send TtsEnd frame to client
```
