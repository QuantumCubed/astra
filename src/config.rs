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

pub fn load_system_prompt() -> Result<String, std::io::Error> {
    let mut prompt = String::new();

    load_dir("config/core", &mut prompt)?;
    load_dir("config/user", &mut prompt)?;

    Ok(prompt)
}