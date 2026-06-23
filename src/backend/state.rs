use any_tts::TtsModel;
use std::sync::Arc;
use whisper_rs::WhisperContext;

use crate::backend::audio::{stt::load_whisper_ctx, tts::load_tts_model};
use crate::backend::config::{
    load_ollama_url, load_system_prompt, load_tts_model_path, load_tts_voice, load_whisper_model_path,
};
use crate::tools::registry::{register_tools, Tool};

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Tool>,
    pub system_prompt: String,
    pub ollama_url: String,
    pub client: reqwest::Client,
    pub whisper_ctx: Arc<WhisperContext>,
    pub tts_model: Arc<dyn TtsModel>,
    pub tts_sample_rate: u32,
    pub tts_voice: String
}

impl AppState {
    pub fn new() -> Self {
        let whisper_path = load_whisper_model_path()
            .expect("failed to load WHISPER_MODEL from .astra/astra.conf");
        let whisper_ctx =
            load_whisper_ctx(&whisper_path).expect("failed to load Whisper STT model");

        let tts_path = load_tts_model_path()
            .expect("failed to load KOKORO_MODEL from .astra/astra.conf");
        let tts_model =
            load_tts_model(&tts_path).expect("failed to load Kokoro TTS model");
        let tts_sample_rate = tts_model.sample_rate();

        let tts_voice = load_tts_voice().unwrap_or_else(|_| "af_heart".to_string());

        Self {
            ollama_url: load_ollama_url()
                .expect("failed to load OLLAMA_ENDPOINT from .astra/astra.conf"),
            tools: register_tools(),
            system_prompt: load_system_prompt().expect("failed to load config files"),
            client: reqwest::Client::new(),
            whisper_ctx,
            tts_model,
            tts_sample_rate,
            tts_voice
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
