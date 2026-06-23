use std::sync::{Arc, Mutex};
use kokoro_tiny::TtsEngine;
use whisper_rs::WhisperContext;

use crate::backend::audio::{stt::load_whisper_ctx, tts::load_tts_engine};
use crate::backend::config::{
    load_ollama_url, load_system_prompt, load_tts_voice, load_whisper_model_path,
};
use crate::tools::registry::{register_tools, Tool};

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Tool>,
    pub system_prompt: String,
    pub ollama_url: String,
    pub client: reqwest::Client,
    pub whisper_ctx: Arc<WhisperContext>,
    pub tts: Arc<Mutex<TtsEngine>>,
    pub tts_voice: String,
}

impl AppState {
    pub async fn new() -> Self {
        let whisper_ctx = tokio::task::spawn_blocking(|| {
            let path = load_whisper_model_path()
                .expect("failed to load WHISPER_MODEL from .astra/astra.conf");
            load_whisper_ctx(&path).expect("failed to load Whisper STT model")
        })
        .await
        .expect("whisper init panicked");

        let tts = load_tts_engine()
            .await
            .expect("failed to load Kokoro TTS engine");

        let tts_voice = load_tts_voice().unwrap_or_else(|_| "af_heart".to_string());

        Self {
            ollama_url: load_ollama_url()
                .expect("failed to load OLLAMA_ENDPOINT from .astra/astra.conf"),
            tools: register_tools(),
            system_prompt: load_system_prompt().expect("failed to load config files"),
            client: reqwest::Client::new(),
            whisper_ctx,
            tts,
            tts_voice,
        }
    }
}
