//! Step 0 spike: does qwen3-tts build and run faster than realtime (RTF < 1) here?
//!
//! Run from THIS directory in WSL2 (release matters for perf):
//!   cd tts-spike && cargo run --release
//!
//! First run auto-downloads the GGUF/ONNX model into ./models and the llama.cpp +
//! onnxruntime binaries into ./runtime (so it will sit a while the first time).
//!
//! RTF = (printed "synthesized in Ns") / (duration of spike_output.wav).
//! Check the wav duration with `ffprobe spike_output.wav` (or play it). We want RTF < 1.
//! Also watch `nvidia-smi` in another terminal — expect ~2 GB.

use qwen3_tts::{TtsEngine, VoiceFile};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t = Instant::now();
    // args: (model_dir, quant, n_threads). n_threads is unused; "none" is the README's
    // default quant — if RTF comes out high, try a GGUF quant like "Q5_K_M" or "Q4_K_M".
    let mut engine = TtsEngine::new("models", "none", 0).await?;
    println!("engine loaded in {:.1}s", t.elapsed().as_secs_f32());

    // If this errors with "file not found", list ./speakers after the first run to see
    // which presets shipped, and swap the name — or create one from a reference clip via
    // `TtsEngine::create_voice_file(ref_wav, ref_text)`. `vivian` is the stated default.
    let voice = VoiceFile::load("speakers/sohee.json")?;

    let text = "I'm Astra, a personal AI assistant running on your home server.";
    let t = Instant::now();
    let audio = engine.generate_with_voice(text, &voice, None)?;
    let synth_s = t.elapsed().as_secs_f32();

    // AudioSample exposes .samples (Vec<f32>) and .sample_rate — compute RTF directly.
    let audio_s = audio.samples.len() as f32 / audio.sample_rate as f32;
    println!(
        "RTF {:.2}  ({audio_s:.1}s audio in {synth_s:.1}s)  — want < 1",
        synth_s / audio_s.max(0.01)
    );

    let _ = audio.save_wav("spike_output.wav");
    Ok(())
}
