use std::sync::Arc;
use any_tts::{DType, DeviceSelection, ModelType, SynthesisRequest, TtsConfig, TtsModel, load_model};

/// Load the Qwen3-TTS model via any-tts.
///
/// Blocking: on the first run this downloads ~4.5 GB of weights from HuggingFace,
/// and any-tts may spin up its own runtime internally while loading. Callers must
/// invoke this inside `spawn_blocking`.
pub fn load_tts_model(
    model_id: Option<String>,
    dtype: Option<String>,
) -> anyhow::Result<Arc<dyn TtsModel>> {
    // `Auto` selects CUDA → Metal → CPU; dtype defaults to BF16 on GPU.
    let mut config = TtsConfig::new(ModelType::Qwen3Tts).with_device(DeviceSelection::Auto);
    // Override the default 1.7B with e.g. `Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice` for
    // lower latency (same architecture, fewer params, same CustomVoice timbres).
    if let Some(id) = model_id {
        config = config.with_hf_model_id(id);
    }
    // Optional dtype override. f16 is the same VRAM as bf16 but may hit a faster
    // matmul kernel on Ampere; f32 doubles VRAM (diagnostic only).
    if let Some(dtype) = dtype {
        let dtype = match dtype.to_ascii_lowercase().as_str() {
            "bf16" => DType::BF16,
            "f16" | "fp16" => DType::F16,
            "f32" | "fp32" => DType::F32,
            other => {
                return Err(anyhow::anyhow!(
                    "unknown TTS_DTYPE '{other}' (expected bf16, f16, or f32)"
                ));
            }
        };
        config = config.with_dtype(dtype);
    }
    let model = load_model(config)
        .map_err(|e| anyhow::anyhow!("failed to load Qwen3-TTS model: {e}"))?;
    tracing::info!("TTS voices available: {:?}", model.supported_voices());
    Ok(Arc::from(model))
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

/// Synthesize one chunk of text into f32 PCM at the model's native sample rate.
///
/// Qwen3-TTS is autoregressive, so synthesis is a heavy blocking call that runs
/// on a blocking thread. The caller streams the returned samples per chunk.
/// `max_tokens` caps generated codec tokens to bound latency and VRAM (any-tts's
/// model default is 2048 ≈ ~170s of audio, far more than a sentence needs).
pub async fn synthesize_sentence(
    tts: Arc<dyn TtsModel>,
    voice: String,
    sentence: String,
    max_tokens: usize,
) -> anyhow::Result<Vec<f32>> {
    tokio::task::spawn_blocking(move || {
        let request = SynthesisRequest::new(sentence)
            .with_language("en")
            .with_voice(voice)
            .with_max_tokens(max_tokens);
        let audio = tts
            .synthesize(&request)
            .map_err(|e| anyhow::anyhow!("TTS synthesis failed: {e}"))?;
        Ok(audio.samples)
    })
    .await?
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
