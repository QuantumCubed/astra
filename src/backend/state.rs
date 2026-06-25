use std::sync::Arc;
use any_tts::TtsModel;
use whisper_rs::WhisperContext;

use crate::backend::audio::{stt::load_whisper_ctx, tts::load_tts_model};
use crate::backend::config::{
    load_ollama_url, load_system_prompt, load_tts_dtype, load_tts_max_tokens, load_tts_model_id,
    load_tts_voice, load_whisper_model_path,
};
use crate::tools::registry::{register_tools, Tool};

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Tool>,
    pub system_prompt: String,
    pub ollama_url: String,
    pub client: reqwest::Client,
    pub whisper_ctx: Arc<WhisperContext>,
    pub tts: Arc<dyn TtsModel>,
    pub tts_voice: String,
    pub tts_max_tokens: usize,
}

impl AppState {
    pub async fn new() -> Self {
        let started = std::time::Instant::now();
        let whisper_ctx = tokio::task::spawn_blocking(|| {
            let path = load_whisper_model_path()
                .expect("failed to load WHISPER_MODEL from .astra/astra.conf");
            load_whisper_ctx(&path).expect("failed to load Whisper STT model")
        })
        .await
        .expect("whisper init panicked");
        tracing::info!("whisper model loaded in {:.1}s", started.elapsed().as_secs_f32());

        // any-tts loading is blocking and creates its own runtime internally, so it
        // must run outside the async context (see Decisions.md, 2026-06-23 WSL2 entry).
        let tts_model_id = load_tts_model_id();
        let tts_dtype = load_tts_dtype();
        let started = std::time::Instant::now();
        let tts = tokio::task::spawn_blocking(move || load_tts_model(tts_model_id, tts_dtype))
            .await
            .expect("tts init panicked")
            .expect("failed to load Qwen3-TTS model");
        tracing::info!("TTS model loaded in {:.1}s", started.elapsed().as_secs_f32());

        let tts_voice = load_tts_voice().unwrap_or_else(|_| "ryan".to_string());
        let tts_max_tokens = load_tts_max_tokens().unwrap_or(512);

        Self {
            ollama_url: load_ollama_url()
                .expect("failed to load OLLAMA_ENDPOINT from .astra/astra.conf"),
            tools: register_tools(),
            system_prompt: load_system_prompt().expect("failed to load config files"),
            client: reqwest::Client::new(),
            whisper_ctx,
            tts,
            tts_voice,
            tts_max_tokens,
        }
    }
}
