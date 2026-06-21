use crate::handlers::req_handler::OllamaMessage;

const CONTEXT_WINDOW: usize = 4096;
const RESPONSE_BUFFER: usize = 512;

pub struct Conversation {
    history: Vec<OllamaMessage>,
    max_tokens: usize
}

impl Conversation {
    pub fn new(system_prompt: String) -> Self {
        Self {
            history: vec![OllamaMessage { role: "system".to_string(), content: system_prompt }], // should I add to_string to sys prompt var?
            max_tokens: CONTEXT_WINDOW - RESPONSE_BUFFER
        }
    }

    pub fn add_user_turn(&mut self, content: &str) {
        self.history.push(OllamaMessage { role: "user".to_string(), content: content.to_string() });
        self.enforce_window();
    }

    pub fn add_astra_turn(&mut self, content: &str) {
        self.history.push(OllamaMessage { role: "assistant".to_string(), content: content.to_string() });
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