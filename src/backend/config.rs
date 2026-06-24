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

/// Max codec tokens per sentence for TTS generation. Returns `None` if the key is
/// absent or unparseable so the caller can apply its own default. Caps runaway
/// autoregressive generation (latency + VRAM); any-tts's model default is 2048.
pub fn load_tts_max_tokens() -> Option<usize> {
    let content = std::fs::read_to_string(".astra/astra.conf").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("TTS_MAX_TOKENS=") {
            return value.trim().parse().ok();
        }
    }
    None
}

/// Optional HuggingFace model id override for the Qwen3-TTS backend (e.g. the 0.6B
/// CustomVoice variant for lower latency). `None` → any-tts's default (1.7B).
pub fn load_tts_model_id() -> Option<String> {
    let content = std::fs::read_to_string(".astra/astra.conf").ok()?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("TTS_MODEL_ID=") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}