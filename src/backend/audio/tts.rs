use std::sync::{Arc, Mutex};
use qwen3_tts::{TtsEngine, VoiceFile};

/// Qwen3-TTS always decodes audio at 24 kHz (hardcoded in the crate's `AudioSample`).
/// The engine exposes no accessor, so we surface it as a constant for the protocol frames.
pub const TTS_SAMPLE_RATE: u32 = 24000;

/// Directory holding the TTS model files (`gguf/`, `onnx/`, `tokenizer/`), mirroring
/// whisper's `.astra/models/stt`. NOTE: the crate separately resolves `runtime/` (the
/// llama.cpp + onnxruntime shared libs) relative to the *process CWD*, so `runtime/`
/// must sit at astra's working directory — it is not under this dir.
const TTS_MODEL_DIR: &str = ".astra/models/tts";

/// Load the Qwen3-TTS engine (GGUF talker/predictor via llama.cpp + ONNX decoder).
///
/// Async because the crate's `new` awaits a model auto-download on first run. Unlike
/// any-tts (which block_on'd internally and so needed `spawn_blocking`), this is a real
/// async fn, so callers can `.await` it directly. `quant` selects the GGUF variant:
/// "none" → `gguf/`, "q5_k_m" → `gguf_q5_k_m/`, "q8_0" → `gguf_q8_0/` (lower-cased
/// defensively, since the crate string-matches it). `max_steps` caps the autoregressive
/// frame count per synthesis (applied once here via `set_max_steps`).
pub async fn load_tts_model(quant: String, max_steps: usize) -> anyhow::Result<TtsEngine> {
    let mut engine = TtsEngine::new(TTS_MODEL_DIR, &quant.to_ascii_lowercase(), 0)
        .await
        .map_err(|e| anyhow::anyhow!("failed to load Qwen3-TTS engine: {e}"))?;
    engine.set_max_steps(max_steps);
    Ok(engine)
}

/// Load a speaker `VoiceFile` (a precomputed codec/embedding profile) by name from
/// `<TTS_MODEL_DIR>/speakers/<name>.json`. Loaded once at startup and shared via `Arc`.
pub fn load_tts_voice_file(name: &str) -> anyhow::Result<VoiceFile> {
    let path = format!("{TTS_MODEL_DIR}/speakers/{name}.json");
    VoiceFile::load(&path)
        .map_err(|e| anyhow::anyhow!("failed to load TTS voice '{name}' from {path}: {e}"))
}

/// Drain complete sentences from a streaming text buffer, leaving any trailing
/// partial sentence in `buf`. A sentence is complete once a `.`/`!`/`?` is followed
/// by whitespace (so we've seen past the boundary), which lets the caller synthesize
/// each sentence the moment it forms instead of waiting for the whole response.
pub fn take_sentences(buf: &mut String) -> Vec<String> {
    let mut sentences = Vec::new();
    loop {
        let bytes = buf.as_bytes();
        let mut boundary = None;
        for i in 0..bytes.len() {
            if matches!(bytes[i], b'.' | b'!' | b'?')
                && bytes.get(i + 1).is_some_and(|b| b.is_ascii_whitespace())
            {
                boundary = Some(i + 1);
                break;
            }
        }
        match boundary {
            // `i + 1` falls on an ASCII whitespace byte, so both slices land on char
            // boundaries even when the sentence contains multi-byte UTF-8.
            Some(idx) => {
                let sentence = buf[..idx].trim().to_string();
                *buf = buf[idx..].trim_start().to_string();
                if !sentence.is_empty() {
                    sentences.push(sentence);
                }
            }
            None => break,
        }
    }
    sentences
}

/// Synthesize one chunk of text, streaming 24 kHz f32 PCM out through the returned
/// channel as the decoder produces it (~every 4 codec frames ≈ 320 ms) — so playback can
/// start long before the whole sentence finishes.
///
/// `generate_with_voice_streaming` is a heavy, blocking, autoregressive FFI call that
/// takes `&mut self`, so it runs on a blocking thread holding the (std) mutex for its
/// duration (one synthesis at a time). It pushes chunks into the tokio `UnboundedSender`
/// from its own decoder thread while running. Returns the receiver to drain, plus the
/// `JoinHandle` to await for completion / error (the channel closes when synthesis ends).
/// The per-sentence frame cap was applied once at load via `set_max_steps`.
pub fn synthesize_sentence_streaming(
    tts: Arc<Mutex<TtsEngine>>,
    voice: Arc<VoiceFile>,
    sentence: String,
) -> (
    tokio::sync::mpsc::UnboundedReceiver<Vec<f32>>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<f32>>();
    let handle = tokio::task::spawn_blocking(move || {
        let mut engine = tts
            .lock()
            .map_err(|_| anyhow::anyhow!("TTS engine mutex poisoned"))?;
        engine
            .generate_with_voice_streaming(&sentence, &voice, None, Some(tx))
            .map_err(|e| anyhow::anyhow!("TTS synthesis failed: {e}"))?;
        Ok(())
    });
    (rx, handle)
}

pub fn strip_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;

    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let t = strip_block_markers(t);
        let t = strip_inline_markers(t);
        let t = strip_unpronounceable(&t);
        if !t.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&t);
        }
    }

    out.trim().to_string()
}

fn strip_block_markers(line: &str) -> &str {
    let s = line.trim_start_matches('#').trim_start();
    let s = s.strip_prefix("> ").unwrap_or(s);
    let s = if s.len() >= 2
        && matches!(s.as_bytes()[0], b'-' | b'*' | b'+')
        && s.as_bytes()[1] == b' '
    {
        &s[2..]
    } else {
        s
    };
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i > 0 && s[i..].starts_with(". ") { &s[i + 2..] } else { s }
}

fn strip_unpronounceable(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;
    for c in s.chars() {
        if c.is_ascii() && !c.is_ascii_control() {
            out.push(c);
            last_space = c == ' ';
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim().to_string()
}

fn strip_inline_markers(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' | '_' => {
                if chars.peek() == Some(&c) {
                    chars.next();
                }
            }
            '`' => {
                while chars.peek() == Some(&'`') {
                    chars.next();
                }
            }
            '[' => {
                let mut link_text = String::new();
                let mut closed = false;
                for lc in chars.by_ref() {
                    if lc == ']' {
                        closed = true;
                        break;
                    }
                    link_text.push(lc);
                }
                if closed && chars.peek() == Some(&'(') {
                    chars.next();
                    let mut depth = 1i32;
                    for lc in chars.by_ref() {
                        match lc {
                            '(' => depth += 1,
                            ')' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    result.push_str(&link_text);
                } else {
                    result.push('[');
                    result.push_str(&link_text);
                    if closed {
                        result.push(']');
                    }
                }
            }
            _ => result.push(c),
        }
    }

    result
}
