use crate::config::load_system_prompt;
use crate::tools::registry::register_tools;
use crate::tools::registry::Tool;

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Tool>,
    pub system_prompt: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tools: register_tools(),
            system_prompt: load_system_prompt().expect("failed to load config files")
        }
    }
}