use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    #[serde(rename = "total_tokens")]
    pub total: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ModelEntry {
    pub id: String,
    pub name: Option<String>,
}

impl Display for ModelEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let label = self.name.as_deref().unwrap_or(&self.id);
        write!(f, "{}", label)
    }
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
    pub usage: Option<TokenUsage>,
}

#[derive(Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessageContent,
}

#[derive(Deserialize)]
pub struct ChatMessageContent {
    pub content: String,
}
