use crate::ai::{GenerateCommitMsg, ListModels, ModelEntry};
use async_trait::async_trait;
use reqwest::Client;

const url: &str = "https://generativelanguage.googleapis.com/v1beta/";
pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
    sys_prompt: String,
}

impl GeminiClient {
    fn new(api_key: String) -> Self {
        let client = Client::new();
        Self {
            client,
            api_key,
            model: String::new(),
            sys_prompt: String::new(),
        }
    }

    fn sys_prompt(mut self, prompt: String) -> Self {
        self.sys_prompt = prompt;
        self
    }

    fn model(mut self, model: ModelEntry) -> Self {
        self.model = model.name;
        self
    }
}

#[async_trait]
impl GenerateCommitMsg for GeminiClient {
    async fn generate_commit_msg(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(String::new())
    }
}

#[async_trait]
impl ListModels for GeminiClient {
    async fn list_models(
        &self,
    ) -> Result<Vec<ModelEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let models: Vec<ModelEntry> = Vec::new();
        Ok(models)
    }
}
