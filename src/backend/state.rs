use crate::backend::config::{load_ollama_url, load_system_prompt, load_whisper_model_path};
use crate::tools::registry::{register_tools, Tool};

#[derive(Clone)]
pub struct AppState {
    pub ollama_url: String,
    pub whisper_model_path: String,
    pub tools: Vec<Tool>,
    pub system_prompt: String,
    pub client: reqwest::Client
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ollama_url: load_ollama_url().expect("failed to load OLLAMA_IP from .astra/astra.conf"),
            whisper_model_path: load_whisper_model_path().expect("failed to load WHISPER_MODEL from .astra/astra.conf"),
            tools: register_tools(),
            system_prompt: load_system_prompt().expect("failed to load config files"),
            client: reqwest::Client::new()
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
