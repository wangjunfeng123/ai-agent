use crate::agent::ContentItem;

#[derive(Debug, Clone, Default)]
pub struct LlmRequest {
    pub instructions: Vec<String>,
    pub constents: Vec<ContentItem>,
}

impl LlmRequest {
    pub fn append_instructioons(&mut self, instructions: impl Into<String>) {
        self.instructions.push(instructions.into());
    }
}
