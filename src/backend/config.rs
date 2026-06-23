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

pub fn load_tts_model_path() -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(".astra/astra.conf")?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("KOKORO_MODEL=") {
            return Ok(value.trim().to_string());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "KOKORO_MODEL key not found in .astra/astra.conf",
    ))
}

pub fn load_tts_voice() -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(".astra/astra.conf")?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("KOKORO_VOICE=") {
            return Ok(value.trim().to_string());
        }
    }
    return Ok("af_heart".to_string())
}