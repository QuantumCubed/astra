use crate::backend::ollama::types::{OllamaMessage, Role};

const CONTEXT_WINDOW: usize = 4096;
const RESPONSE_BUFFER: usize = 512;

pub struct Conversation {
    history: Vec<OllamaMessage>,
    max_tokens: usize,
}

impl Conversation {
    pub fn new(system_prompt: &str) -> Self {
        Self {
            history: vec![OllamaMessage {
                role: Role::System,
                content: system_prompt.to_string(),
                tool_calls: None,
            }],
            max_tokens: CONTEXT_WINDOW - RESPONSE_BUFFER,
        }
    }

    pub fn add_astra_tool_call(&mut self, tool_calls: Vec<serde_json::Value>) {
        self.history.push(OllamaMessage {
            role: Role::Assistant,
            content: String::new(),
            tool_calls: Some(tool_calls),
        });
    }

    pub fn add_tool_result(&mut self, content: &str) {
        self.history.push(OllamaMessage {
            role: Role::Tool,
            content: content.to_string(),
            tool_calls: None,
        });
    }

    pub fn add_user_turn(&mut self, content: &str) {
        self.history.push(OllamaMessage {
            role: Role::User,
            content: content.to_string(),
            tool_calls: None,
        });
        self.enforce_window();
    }

    pub fn add_astra_turn(&mut self, content: &str) {
        self.history.push(OllamaMessage {
            role: Role::Assistant,
            content: content.to_string(),
            tool_calls: None,
        });
        self.enforce_window();
    }

    pub fn messages(&self) -> &[OllamaMessage] {
        &self.history
    }

    fn estimate_tokens(&self) -> usize {
        self.history
            .iter()
            .map(|msg| msg.content.chars().count() / 4)
            .sum()
    }

    fn enforce_window(&mut self) {
        while self.estimate_tokens() >= self.max_tokens && self.history.len() > 1 {
            self.history.remove(1);
        }
    }
}
