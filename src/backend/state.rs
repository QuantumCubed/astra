use std::sync::{Arc, Mutex};
use qwen3_tts::{TtsEngine, VoiceFile};
use whisper_rs::WhisperContext;

use crate::backend::audio::{
    stt::load_whisper_ctx,
    tts::{load_tts_model, load_tts_voice_file},
};
use crate::backend::config::{
    load_conf, load_system_prompt, ollama_model, ollama_url, tts_max_tokens, tts_quant, tts_voice,
    whisper_model_path,
};
use crate::tools::registry::{register_tools, Tool};

#[derive(Clone)]
pub struct AppState {
    // Read-only after startup and shared across every connection, so the heavy fields sit
    // behind `Arc` — cloning `AppState` per connection is then just pointer bumps.
    pub tools: Arc<Vec<Tool>>,
    pub system_prompt: Arc<str>,
    pub ollama_url: String,
    pub model: String,
    pub client: reqwest::Client,
    pub whisper_ctx: Arc<WhisperContext>,
    // generate_with_voice takes &mut self, so the engine lives behind a Mutex; the
    // blocking synthesis holds the lock on a spawn_blocking thread (see synthesize_sentence_streaming).
    pub tts: Arc<Mutex<TtsEngine>>,
    // Speaker profile, loaded once at startup and shared read-only.
    pub tts_voice: Arc<VoiceFile>,
}

impl AppState {
    pub async fn new() -> Self {
        let conf = load_conf();

        let whisper_path = whisper_model_path(&conf)
            .expect("WHISPER_MODEL missing from .astra/astra.conf")
            .to_string();
        let started = std::time::Instant::now();
        let whisper_ctx = tokio::task::spawn_blocking(move || {
            load_whisper_ctx(&whisper_path).expect("failed to load Whisper STT model")
        })
        .await
        .expect("whisper init panicked");
        tracing::info!("whisper model loaded in {:.1}s", started.elapsed().as_secs_f32());

        // qwen3-tts's `new` is genuinely async (it awaits a model auto-download), so unlike
        // any-tts we await it directly — no nested-runtime workaround. The synchronous model
        // load it then does briefly blocks this startup worker, which is fine before we serve.
        let tts_quant = tts_quant(&conf).unwrap_or("none").to_string();
        let tts_max_steps = tts_max_tokens(&conf).unwrap_or(512);
        let started = std::time::Instant::now();
        let tts = load_tts_model(tts_quant, tts_max_steps)
            .await
            .expect("failed to load Qwen3-TTS engine");
        tracing::info!("TTS model loaded in {:.1}s", started.elapsed().as_secs_f32());

        let voice_name = tts_voice(&conf).unwrap_or("ryan");
        let voice = load_tts_voice_file(voice_name)
            .unwrap_or_else(|e| panic!("failed to load TTS voice '{voice_name}': {e}"));

        Self {
            ollama_url: ollama_url(&conf)
                .expect("OLLAMA_ENDPOINT missing from .astra/astra.conf")
                .to_string(),
            model: ollama_model(&conf).unwrap_or("qwen3.5:9b").to_string(),
            tools: Arc::new(register_tools()),
            system_prompt: load_system_prompt()
                .expect("failed to load config files")
                .into(),
            client: reqwest::Client::new(),
            whisper_ctx,
            tts: Arc::new(Mutex::new(tts)),
            tts_voice: Arc::new(voice),
        }
    }
}
