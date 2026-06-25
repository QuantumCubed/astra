use std::sync::{Arc, Mutex};
use qwen3_tts::{TtsEngine, VoiceFile};
use whisper_rs::WhisperContext;

use crate::backend::audio::{
    stt::load_whisper_ctx,
    tts::{load_tts_model, load_tts_voice_file},
};
use crate::backend::config::{
    load_ollama_url, load_system_prompt, load_tts_max_tokens, load_tts_quant, load_tts_voice,
    load_whisper_model_path,
};
use crate::tools::registry::{register_tools, Tool};

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Tool>,
    pub system_prompt: String,
    pub ollama_url: String,
    pub client: reqwest::Client,
    pub whisper_ctx: Arc<WhisperContext>,
    // generate_with_voice takes &mut self, so the engine lives behind a Mutex; the
    // blocking synthesis holds the lock on a spawn_blocking thread (see synthesize_sentence).
    pub tts: Arc<Mutex<TtsEngine>>,
    // Speaker profile, loaded once at startup and shared read-only.
    pub tts_voice: Arc<VoiceFile>,
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

        // qwen3-tts's `new` is genuinely async (it awaits a model auto-download), so unlike
        // any-tts we await it directly — no nested-runtime workaround. The synchronous model
        // load it then does briefly blocks this startup worker, which is fine before we serve.
        let tts_quant = load_tts_quant().unwrap_or_else(|| "none".to_string());
        let tts_max_steps = load_tts_max_tokens().unwrap_or(512);
        let started = std::time::Instant::now();
        let tts = load_tts_model(tts_quant, tts_max_steps)
            .await
            .expect("failed to load Qwen3-TTS engine");
        tracing::info!("TTS model loaded in {:.1}s", started.elapsed().as_secs_f32());

        let tts_voice_name = load_tts_voice().unwrap_or_else(|_| "ryan".to_string());
        let tts_voice = load_tts_voice_file(&tts_voice_name)
            .unwrap_or_else(|e| panic!("failed to load TTS voice '{tts_voice_name}': {e}"));

        Self {
            ollama_url: load_ollama_url()
                .expect("failed to load OLLAMA_ENDPOINT from .astra/astra.conf"),
            tools: register_tools(),
            system_prompt: load_system_prompt().expect("failed to load config files"),
            client: reqwest::Client::new(),
            whisper_ctx,
            tts: Arc::new(Mutex::new(tts)),
            tts_voice: Arc::new(tts_voice),
        }
    }
}
