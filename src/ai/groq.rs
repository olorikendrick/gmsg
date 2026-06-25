// src/ai/gemini.rs
use crate::ai::types::{
    ChatMessage, ChatRequest, ChatResponse, Message, ModelEntry, Role, TokenUsage,
};
use crate::ai::{CompletionClient, ModelProvider};
use async_trait::async_trait;
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

const BASE_URL: &str = "https://api.groq.com/openai/v1/";

pub struct GroqProvider {
    api_key: String,
    client: Client,
}

impl GroqProvider {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            api_key: std::env::var("GROQ_API_KEY")?,
            client: Client::new(),
        })
    }
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<GroqModel>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GroqModel {
    id: String,
}

#[async_trait]
impl ModelProvider for GroqProvider {
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
            .map(|m| ModelEntry {
                id: m.id.to_string(),
                name: None,
            })
            .collect())
    }

    fn create_completion_client(
        &self,
        model: ModelEntry,
        sys_prompt: String,
    ) -> anyhow::Result<Box<dyn CompletionClient>> {
        Ok(Box::new(GroqClient {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            model: model.id,
            sys_prompt,
        }))
    }
}

pub struct GroqClient {
    client: Client,
    api_key: String,
    model: String,
    sys_prompt: String,
}

#[async_trait]
impl CompletionClient for GroqClient {
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

        let chat: ChatResponse = self
            .client
            .post(format!("{}chat/completions", BASE_URL))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&ChatRequest {
                model: self.model.clone(),
                messages: chat_messages,
            })
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
            .ok_or_else(|| anyhow::anyhow!("Groq returned no choices"))?;

        Ok((text, chat.usage.unwrap_or_default()))
    }
}
