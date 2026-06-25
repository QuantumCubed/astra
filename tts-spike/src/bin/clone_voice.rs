use qwen3_tts::TtsEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let ref_audio = args.next().expect("usage: clone-voice <ref.wav> <ref-text> <out.json>");
    let ref_text = args.next().expect("usage: clone-voice <ref.wav> <ref-text> <out.json>");
    let out_path = args.next().expect("usage: clone-voice <ref.wav> <ref-text> <out.json>");

    println!("Loading TTS engine...");
    let mut engine = TtsEngine::new("models", "Q5_K_M", 0).await?;

    println!("Creating voice file from {ref_audio}...");
    let voice = engine.create_voice_file(&ref_audio, ref_text)?;
    voice.save(&out_path).map_err(|e| anyhow::anyhow!(e))?;

    println!("Saved voice to {out_path}");
    Ok(())
}
