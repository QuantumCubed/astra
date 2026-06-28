use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use qwen3_tts::{TtsEngine, VoiceFile};
use whisper_rs::WhisperContext;

use crate::{backend::audio::{
    stt::load_whisper_ctx,
    tts::{load_tts_model, load_tts_voice_file},
}, integrations::spotify::spotify_connection::{refresh_access_token, get_devices}};
use crate::backend::config::{
    load_conf, load_system_prompt, ollama_model, ollama_url, tts_max_tokens, tts_quant, tts_voice,
    whisper_model_path, spotify_client_id, spotify_client_secret, spotify_refresh_token
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
    pub spotify_token: Arc<tokio::sync::Mutex<String>>,
    // generate_with_voice takes &mut self, so the engine lives behind a Mutex; the
    // blocking synthesis holds the lock on a spawn_blocking thread (see synthesize_sentence_streaming).
    pub tts: Arc<Mutex<TtsEngine>>,
    // Speaker profile, loaded once at startup and shared read-only.
    pub tts_voice: Arc<VoiceFile>,
    pub spotify_devices: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
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

        let spotify_client_id = spotify_client_id(&conf).expect("SPOTIFY_CLIENT_ID missing from .astra/astra.conf");
        let spotify_client_secret = spotify_client_secret(&conf).expect("SPOTIFY_CLIENT_SECRET missing from .astra/astra.conf");
        let spotify_refresh_token = spotify_refresh_token(&conf).expect("SPOTIFY_REFRESH_TOKEN missing from .astra/astra.conf");
        let client = reqwest::Client::new();
        let spotify_token = refresh_access_token(&client, spotify_client_id, spotify_client_secret, spotify_refresh_token)
            .await
            .expect("failed to fetch Spotify access token");
        let spotify_devices = get_devices(&client, &spotify_token)
            .await
            .unwrap_or_default();

        Self {
            ollama_url: ollama_url(&conf)
                .expect("OLLAMA_ENDPOINT missing from .astra/astra.conf")
                .to_string(),
            model: ollama_model(&conf).unwrap_or("qwen3.5:9b").to_string(),
            tools: Arc::new(register_tools()),
            system_prompt: load_system_prompt()
                .expect("failed to load config files")
                .into(),
            whisper_ctx,
            tts: Arc::new(Mutex::new(tts)),
            tts_voice: Arc::new(voice),
            client,
            spotify_token: Arc::new(tokio::sync::Mutex::new(spotify_token)),
            spotify_devices: Arc::new(tokio::sync::Mutex::new(spotify_devices)),
        }
    }
}
