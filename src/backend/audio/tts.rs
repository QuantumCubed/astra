use std::sync::{Arc, Mutex};
use kokoro_tiny::TtsEngine;

pub async fn load_tts_engine() -> anyhow::Result<Arc<Mutex<TtsEngine>>> {
    let engine = TtsEngine::new()
        .await
        .map_err(|e| anyhow::anyhow!("failed to load Kokoro model: {e}"))?;
    Ok(Arc::new(Mutex::new(engine)))
}

pub async fn synthesize(
    tts: Arc<Mutex<TtsEngine>>,
    voice: String,
    text: String,
) -> anyhow::Result<(Vec<f32>, u32)> {
    let text = strip_markdown(&text);
    tokio::task::spawn_blocking(move || {
        let mut engine = tts
            .lock()
            .map_err(|_| anyhow::anyhow!("TTS engine lock poisoned"))?;
        let mut all_samples = Vec::new();
        for sentence in split_sentences(&text) {
            let samples = engine
                .synthesize(&sentence, Some(&voice))
                .map_err(|e| anyhow::anyhow!("TTS synthesis failed: {e}"))?;
            all_samples.extend(samples);
        }
        Ok((all_samples, 24000u32))
    })
    .await?
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();

    for i in 0..bytes.len() {
        if matches!(bytes[i], b'.' | b'!' | b'?')
            && (i + 1 == bytes.len() || bytes[i + 1] == b' ')
        {
            let s = text[start..=i].trim().to_string();
            if !s.is_empty() {
                sentences.push(s);
            }
            start = (i + 2).min(bytes.len());
        }
    }

    if start < text.len() {
        let s = text[start..].trim().to_string();
        if !s.is_empty() {
            sentences.push(s);
        }
    }

    if sentences.is_empty() {
        let s = text.trim().to_string();
        if !s.is_empty() {
            sentences.push(s);
        }
    }

    sentences
}

fn strip_markdown(text: &str) -> String {
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
