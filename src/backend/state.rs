use crate::backend::config::{load_ollama_url, load_system_prompt};
use crate::tools::registry::{register_tools, Tool};

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Tool>,
    pub system_prompt: String,
    pub ollama_url: String,
    pub client: reqwest::Client,
}

impl AppState {
    pub fn new() -> Self {
        let ollama_url = load_ollama_url().expect("failed to load OLLAMA_IP from .astra/astra.conf");
        Self {
            tools: register_tools(),
            system_prompt: load_system_prompt().expect("failed to load config files"),
            ollama_url,
            client: reqwest::Client::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
