use crate::handlers::req_handler::OllamaMessage;

pub struct Conversation {
    history: Vec<OllamaMessage>,
    max_tokens: usize
}

impl Conversation {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            history: Vec::new(),
            max_tokens
        }
    }

    fn estimate_tokens(&self) -> usize {
        self.history
        .iter()
        .map(|msg| msg.content.chars().count() / 4)
        .sum()
    }

    fn enforce_window(&mut self) {
        while self.estimate_tokens() >= self.max_tokens && self.history.len() > 1 {
            self.history.remove(0);
        }
    }
}