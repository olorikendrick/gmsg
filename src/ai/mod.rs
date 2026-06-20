

//! Defines traits and types for generating commit messages via LLM providers.
//!
//! The main entry point is [`Provider`], which initializes a [`ModelProvider`]
//! from environment variables.
//!
//! # Usage
//!
//! ```
//! let provider = Provider::Gemini.initialize()?;
//! let models = provider.list_models().await?;
//! ```


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

///Struct to track token usage per LLM call
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total: u64,
}

///An abstraction over an LLM model
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
    ///List models and creates a completion client
    ///
    ///# Errors 
    ///
    ///May fail due to network errors or api errors
    ///
    ///Returns a [`Vec`] of [`ModelEntry`] items
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>>;
    ///Creates a [`CompletionClient`] from provided [`ModelEntry`]
    ///
    ///# Errors
    ///
    ///May fail due to misconfigured or invalid api key or invalid model
    /// rerurns a boxed [`CompletionClient`]
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
