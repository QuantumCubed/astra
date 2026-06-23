use std::sync::Arc;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub fn load_whisper_ctx(model_path: &str) -> anyhow::Result<Arc<WhisperContext>> {
    let mut params = WhisperContextParameters::default();
    params.use_gpu(true);
    let ctx = WhisperContext::new_with_params(model_path, params)
        .map_err(|e| anyhow::anyhow!("failed to load whisper model: {e}"))?;
    Ok(Arc::new(ctx))
}

pub async fn transcribe(ctx: Arc<WhisperContext>, audio: Vec<u8>) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
        // Raw bytes are 16kHz mono 16-bit little-endian PCM — convert to i16 first
        let samples_i16: Vec<i16> = audio
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();

        // whisper.cpp expects f32 in [-1.0, 1.0]
        let mut samples_f32 = vec![0.0f32; samples_i16.len()];
        whisper_rs::convert_integer_to_float_audio(&samples_i16, &mut samples_f32)
            .map_err(|e| anyhow::anyhow!("audio conversion: {e}"))?;

        // Each transcription gets its own state — no lock needed on the shared context
        let mut state = ctx
            .create_state()
            .map_err(|e| anyhow::anyhow!("whisper create_state: {e}"))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, &samples_f32)
            .map_err(|e| anyhow::anyhow!("whisper inference: {e}"))?;

        let mut transcript = String::new();
        for seg in state.as_iter() {
            let text = seg
                .to_str_lossy()
                .map_err(|e| anyhow::anyhow!("whisper segment text: {e}"))?;
            transcript.push_str(&text);
        }

        Ok(transcript.trim().to_string())
    })
    .await?
}
