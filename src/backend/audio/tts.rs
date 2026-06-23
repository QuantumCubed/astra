use any_tts::{load_model, ModelType, SynthesisRequest, TtsConfig, TtsModel};
use std::sync::Arc;

pub fn load_tts_model(model_path: &str) -> anyhow::Result<Arc<dyn TtsModel>> {
    let model = load_model(TtsConfig::new(ModelType::Kokoro).with_model_path(model_path))?;
    Ok(Arc::from(model))
}

pub async fn synthesize(model: Arc<dyn TtsModel>, voice: &str, text: String) -> anyhow::Result<Vec<f32>> {
    let voice = voice.to_string();
    let text = strip_markdown(&text);
    tokio::task::spawn_blocking(move || {
        let audio = model.synthesize(
            &SynthesisRequest::new(&text).with_language("en").with_voice(&voice),
        )?;
        Ok(audio.samples)
    })
    .await?
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
        if !t.is_empty() {
            out.push_str(&t);
            out.push('\n');
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
    // Strip numbered list prefix: "1. text" → "text"
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i > 0 && s[i..].starts_with(". ") { &s[i + 2..] } else { s }
}

fn strip_inline_markers(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' | '_' => {
                // Skip double marker (** or __) too
                if chars.peek() == Some(&c) {
                    chars.next();
                }
            }
            '`' => {
                // Skip all consecutive backticks (inline code opener/closer)
                while chars.peek() == Some(&'`') {
                    chars.next();
                }
            }
            '[' => {
                // [text](url) → text; otherwise pass through
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
                    chars.next(); // consume '('
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
