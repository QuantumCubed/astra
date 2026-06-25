use std::collections::HashMap;

/// Parse `.astra/astra.conf` once into a `KEY → VALUE` map. A missing file yields an empty
/// map — callers decide which keys are required. Blank lines, `#` comments, lines without
/// `=`, and keys with empty values are skipped. Reading once here replaces the previous
/// per-key re-read + re-scan of the file.
pub fn load_conf() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(content) = std::fs::read_to_string(".astra/astra.conf") else {
        return map;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            if !value.is_empty() {
                map.insert(key.trim().to_string(), value.to_string());
            }
        }
    }
    map
}

/// Ollama server URL (`OLLAMA_ENDPOINT`). Required.
pub fn ollama_url(conf: &HashMap<String, String>) -> Option<&str> {
    conf.get("OLLAMA_ENDPOINT").map(String::as_str)
}

/// Ollama model tag (`OLLAMA_MODEL`). Caller applies a default if absent.
pub fn ollama_model(conf: &HashMap<String, String>) -> Option<&str> {
    conf.get("OLLAMA_MODEL").map(String::as_str)
}

/// Whisper STT model path (`WHISPER_MODEL`). Required.
pub fn whisper_model_path(conf: &HashMap<String, String>) -> Option<&str> {
    conf.get("WHISPER_MODEL").map(String::as_str)
}

/// TTS speaker name (`TTS_VOICE`). Caller applies a default if absent.
pub fn tts_voice(conf: &HashMap<String, String>) -> Option<&str> {
    conf.get("TTS_VOICE").map(String::as_str)
}

/// GGUF quant variant for the Qwen3-TTS talker/predictor (`TTS_QUANT`): `none` (default,
/// `gguf/`), `q5_k_m` (`gguf_q5_k_m/`), or `q8_0` (`gguf_q8_0/`). Quant barely moves RTF
/// (the model isn't memory-bound) but q5_k_m saves ~1.7 GB VRAM.
pub fn tts_quant(conf: &HashMap<String, String>) -> Option<&str> {
    conf.get("TTS_QUANT").map(String::as_str)
}

/// Max generation steps (codec frames, ~80 ms each) per sentence (`TTS_MAX_TOKENS`); maps
/// onto the engine's `set_max_steps`, capping runaway autoregressive generation.
pub fn tts_max_tokens(conf: &HashMap<String, String>) -> Option<usize> {
    conf.get("TTS_MAX_TOKENS").and_then(|v| v.parse().ok())
}

fn load_dir(path: &str, output: &mut String) -> Result<(), std::io::Error> {
    let mut entries: Vec<_> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .collect();

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let content = std::fs::read_to_string(entry.path())?;
        output.push_str(&content);
        output.push('\n');
    }

    Ok(())
}

/// Assemble the system prompt from `.astra/core/*.md` then `.astra/user/*.md`.
pub fn load_system_prompt() -> Result<String, std::io::Error> {
    let mut prompt = String::new();
    load_dir(".astra/core", &mut prompt)?;
    load_dir(".astra/user", &mut prompt)?;
    Ok(prompt)
}
