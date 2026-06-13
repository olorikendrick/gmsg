use crate::ai::{CompletionClient, Message, ModelEntry, ModelProvider, Role, TokenUsage};
use async_trait::async_trait;
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const BASE_URL: &str = "https://api.mistral.ai/v1/";

struct MistralConfig {
    api_key: String,
    client: Client,
}

impl MistralConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("MISTRAL_API_KEY")?;
        let client = Client::new();
        Ok(Self { api_key, client })
    }
}

pub struct MistralProvider {
    config: Arc<MistralConfig>,
}

impl MistralProvider {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Arc::new(MistralConfig::from_env()?);
        Ok(Self { config })
    }
}

#[derive(Deserialize)]
struct MistralModelsResponse {
    data: Vec<MistralModel>,
}

#[derive(Deserialize)]
struct MistralModel {
    id: String,
    name: Option<String>,
    capabilities: MistralCapabilities,
}

#[derive(Deserialize)]
struct MistralCapabilities {
    completion_chat: bool,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    usage: Option<UsageResponse>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct UsageResponse {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

#[async_trait]
impl ModelProvider for MistralProvider {
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>> {
        let endpoint = format!("{}models", BASE_URL);
        let authorization = format!("Bearer {}", self.config.api_key);

        let response = self
            .config
            .client
            .get(endpoint)
            .header(AUTHORIZATION, authorization)
            .send()
            .await?;

        let body: MistralModelsResponse = response.json().await?;

        let models = body
            .data
            .into_iter()
            .filter(|m| m.capabilities.completion_chat)
            .map(|m| ModelEntry {
                id: m.id,
                name: m.name,
            })
            .collect();

        Ok(models)
    }

    fn create_completion_client(
        &self,
        model: ModelEntry,
        sys_prompt: String,
    ) -> anyhow::Result<Box<dyn CompletionClient>> {
        Ok(Box::new(MistralClient {
            config: Arc::clone(&self.config),
            model: model.id,
            sys_prompt,
        }))
    }
}

pub struct MistralClient {
    config: Arc<MistralConfig>,
    model: String,
    sys_prompt: String,
}

#[async_trait]
impl CompletionClient for MistralClient {
    async fn generate_commit_msg(&self, diff: &str) -> anyhow::Result<(String, TokenUsage)> {
        let messages = vec![Message {
            role: Role::User,
            content: format!("Generate a concise git commit message for this diff:\n\n{diff}"),
        }];
        self.prompt(&messages).await
    }

    async fn prompt(&self, messages: &[Message]) -> anyhow::Result<(String, TokenUsage)> {
        let endpoint = format!("{}chat/completions", BASE_URL);
        let authorization = format!("Bearer {}", self.config.api_key);

        let mut chat_messages = vec![ChatMessage {
            role: "system".to_string(),
            content: self.sys_prompt.clone(),
        }];

        chat_messages.extend(messages.iter().map(|m| {
            ChatMessage {
                role: match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                }
                .to_string(),
                content: m.content.clone(),
            }
        }));

        let body = ChatRequest {
            model: self.model.clone(),
            messages: chat_messages,
        };

        let response = self
            .config
            .client
            .post(endpoint)
            .header(AUTHORIZATION, authorization)
            .json(&body)
            .send()
            .await?;

        let chat: ChatResponse = response.json().await?;

        let text = chat
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        let usage = chat
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total: u.total_tokens,
            })
            .unwrap_or_default();

        Ok((text, usage))
    }
}
