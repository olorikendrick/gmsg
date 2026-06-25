// src/ai/mistral.rs
use crate::ai::types::{
    ChatMessage, ChatRequest, ChatResponse, Message, ModelEntry, Role, TokenUsage,
};
use crate::ai::{CompletionClient, ModelProvider};
use async_trait::async_trait;
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

const BASE_URL: &str = "https://api.mistral.ai/v1/";

pub struct MistralProvider {
    api_key: String,
    client: Client,
}

impl MistralProvider {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            api_key: std::env::var("MISTRAL_API_KEY")?,
            client: Client::new(),
        })
    }
}

#[derive(Deserialize)]
struct ModelsResponse {
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

#[async_trait]
impl ModelProvider for MistralProvider {
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>> {
        let body: ModelsResponse = self
            .client
            .get(format!("{}models", BASE_URL))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(body
            .data
            .into_iter()
            .filter(|m| m.capabilities.completion_chat)
            .map(|m| ModelEntry {
                id: m.id,
                name: m.name,
            })
            .collect())
    }

    fn create_completion_client(
        &self,
        model: ModelEntry,
        sys_prompt: String,
    ) -> anyhow::Result<Box<dyn CompletionClient>> {
        Ok(Box::new(MistralClient {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            model: model.id,
            sys_prompt,
        }))
    }
}

pub struct MistralClient {
    client: Client,
    api_key: String,
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

        let chat: ChatResponse = self
            .client
            .post(format!("{}chat/completions", BASE_URL))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let text = chat
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("Mistral returned no choices"))?;

        let usage = chat.usage.unwrap_or_default();

        Ok((text, usage))
    }
}
