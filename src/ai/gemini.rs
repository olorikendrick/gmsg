//! Gemini implementation of [`ModelProvider`] and [`CompletionClient`].
//!
//! # Usage
//!
//! Create a provider from an API key or environment variables:
//!
//! ```
//! let provider = GeminiProvider::from_env()?;
//! ```
use crate::ai::{CompletionClient, Message, ModelEntry, ModelProvider, TokenUsage};
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";

///Reusable configuration to setup connection

struct GeminiConfig {
    client: Client,
    api_key: String,
}

impl GeminiConfig {
    ///Sets up a new Configuration from environment variables
    ///
    ///# Errors
    ///
    ///Returns an error if the `GEMINI_API_KEY` environment variable is not set
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY")?;

        let client = reqwest::Client::new();
        Ok(Self { api_key, client })
    }
}

///Represents an Abstraction over Gemini for listing models
pub struct GeminiProvider {
    config: Arc<GeminiConfig>,
}

impl GeminiProvider {
    ///Creates a new provider from provided API KEY
    ///
    ///This does not check for validity of API KEY
    pub fn new(api_key: String) -> Self {
        let config = Arc::new(GeminiConfig {
            client: Client::new(),
            api_key,
        });
        Self { config }
    }
    ///Creates a new Provider from environment variables
    ///
    ///#Error Returns an error if `GEMINI_API_KEY` is not set in environment
    ///
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Arc::new(GeminiConfig::from_env()?);
        Ok(Self { config })
    }
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>> {
        Ok(Vec::new())
    }

    fn create_completion_client(
        &self,
        model: ModelEntry,
        sys_prompt: String,
    ) -> anyhow::Result<Box<dyn CompletionClient>> {
        Ok(Box::new(GeminiClient {
            config: Arc::clone(&self.config),
            model: model.id,
            sys_prompt,
        }))
    }
}

pub struct GeminiClient {
    config: Arc<GeminiConfig>,
    model: String,
    sys_prompt: String,
}

#[async_trait]
impl CompletionClient for GeminiClient {
    async fn generate_commit_msg(&self, _prompt: &str) -> anyhow::Result<(String, TokenUsage)> {
        let output = (String::new(), TokenUsage::default());
        Ok(output)
    }

    async fn prompt(&self, _messages: &[Message]) -> anyhow::Result<(String, TokenUsage)> {
        let output = (String::new(), TokenUsage::default());
        Ok(output)
    }
}
