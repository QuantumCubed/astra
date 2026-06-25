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

pub fn load_ollama_url() -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(".astra/astra.conf")?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("OLLAMA_ENDPOINT=") {
            return Ok(value.trim().to_string());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "OLLAMA_ENDPOINT key not found in .astra/astra.conf",
    ))
}

pub fn load_system_prompt() -> Result<String, std::io::Error> {
    let mut prompt = String::new();

    load_dir(".astra/core", &mut prompt)?;
    load_dir(".astra/user", &mut prompt)?;

    Ok(prompt)
}

pub fn load_whisper_model_path() -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(".astra/astra.conf")?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("WHISPER_MODEL=") {
            return Ok(value.trim().to_string());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "WHISPER_MODEL key not found in .astra/astra.conf",
    ))
}

pub fn load_tts_voice() -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(".astra/astra.conf")?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("TTS_VOICE=") {
            return Ok(value.trim().to_string());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "TTS_VOICE key not found in .astra/astra.conf",
    ))
}

/// Max generation steps (codec frames, ~80 ms each) per sentence for TTS. Returns
/// `None` if the key is absent or unparseable so the caller can apply its own default.
/// Caps runaway autoregressive generation; maps onto the engine's `set_max_steps`
/// (crate default 4096 ≈ 5 min of audio — far more than a sentence needs).
pub fn load_tts_max_tokens() -> Option<usize> {
    let content = std::fs::read_to_string(".astra/astra.conf").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("TTS_MAX_TOKENS=") {
            return value.trim().parse().ok();
        }
    }
    None
}

/// Optional GGUF quantization variant for the Qwen3-TTS talker/predictor:
/// `none` (default, loads `gguf/`), `q5_k_m` (`gguf_q5_k_m/`), or `q8_0` (`gguf_q8_0/`).
/// Quant barely moved RTF on our GPU (the model isn't memory-bound) but Q5 saves ~1.7 GB
/// VRAM for coexisting with the Ollama LLM. `None` → caller defaults to `none`.
pub fn load_tts_quant() -> Option<String> {
    let content = std::fs::read_to_string(".astra/astra.conf").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("TTS_QUANT=") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}