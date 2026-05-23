// src/state.rs
use crate::tools::registry::register_tools;
use crate::tools::registry::Tool;

#[derive(Clone)]
pub struct AppState {
    pub tools: Vec<Tool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tools: register_tools(),
        }
    }
}