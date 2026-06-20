// src/ai/mod.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

pub mod gemini;
pub mod mistral;
pub mod types;

pub use types::*;

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
            Provider::Gemini => Ok(Box::new(gemini::GeminiProvider::from_env()?)),
            Provider::Mistral => Ok(Box::new(mistral::MistralProvider::from_env()?)),
        }
    }
}
