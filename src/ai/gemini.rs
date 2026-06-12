use crate::ai::{CompletionClient, ModelEntry, ModelProvider};
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";

struct GeminiConfig {
    client: Client,
    api_key: String,
}

pub struct GeminiProvider {
    config: Arc<GeminiConfig>,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        let config = Arc::new(GeminiConfig {
            client: Client::new(),
            api_key,
        });
        Self { config }
    }
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>> {
        Ok(Vec::new())
    }

    fn into_completion_client(
        &self,
        model: ModelEntry,
        sys_prompt: String,
    ) -> anyhow::Result<Box<dyn CompletionClient>> {
        Ok(Box::new(GeminiClient {
            config: Arc::clone(&self.config),
            model: model.name,
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
    async fn generate_commit_msg(&self, _prompt: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }

    async fn prompt(&self, _prompt: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }
}
