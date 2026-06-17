use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};
pub mod gemini;
pub mod mistral;

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

#[async_trait]
pub trait CompletionClient: Send + Sync {
    async fn generate_commit_msg(&self, diff: &str) -> anyhow::Result<(String, TokenUsage)>;
    async fn prompt(&self, messages: &[Message]) -> anyhow::Result<(String, TokenUsage)>;
}

#[async_trait]
pub trait ModelProvider {
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>>;
    fn create_completion_client(
        &self,
        model: ModelEntry,
        sys_prompt: String,
    ) -> anyhow::Result<Box<dyn CompletionClient>>;
}

#[derive(Clone, Serialize, Deserialize, Debug, EnumString, EnumIter, Display)]
pub enum Provider {
    Gemini,
    Mistral,
}

impl Provider {
    pub fn initialize(&self) -> anyhow::Result<Box<dyn ModelProvider>> {
        match self {
            Provider::Gemini => {
                use gemini::GeminiProvider;
                Ok(Box::new(GeminiProvider::from_env()?))
            }
            Provider::Mistral => {
                use mistral::MistralProvider;
                Ok(Box::new(MistralProvider::from_env()?))
            }
        }
    }
}
