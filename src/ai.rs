use http::status::StatusCode;
use rig::{
    agent::{Agent, PromptHook},
    client::{CompletionClient, ModelListingClient, ProviderClient, ProviderClientError},
    completion::{CompletionError, CompletionModel, Prompt, PromptError},
    http_client::Error as HttpError,
    providers::{anthropic, cohere, gemini, ollama, openai, openrouter},
};

use serde::{Deserialize, Serialize};
use strum::EnumIter;
use strum_macros::{Display, EnumString};
pub struct ModelEntry {
    pub display: String,
    pub id: String,
}

#[async_trait::async_trait]
pub trait GenerateCommitMsg {
    async fn generate_commit_msg(&self, diff: &str) -> Result<String, AiError>;
}

#[async_trait::async_trait]
impl<M, P> GenerateCommitMsg for Agent<M, P>
where
    M: CompletionModel + 'static,
    P: PromptHook<M> + 'static,
{
    async fn generate_commit_msg(&self, diff: &str) -> Result<String, AiError> {
        Ok(self.prompt(diff).await?)
    }
}
pub fn build_commit_agent(
    provider: Provider,
    model: String,
    system_message: Option<&str>,
) -> Result<Box<dyn GenerateCommitMsg>, AiError> {
    let preamble = system_message.unwrap();

    let agent: Box<dyn GenerateCommitMsg> = match provider {
        Provider::OpenAI => Box::new(
            openai::Client::from_env()?
                .agent(&model)
                .preamble(preamble)
                .build(),
        ),
        Provider::Gemini => Box::new(
            gemini::Client::from_env()?
                .agent(&model)
                .preamble(preamble)
                .build(),
        ),
        Provider::Anthropic => Box::new(
            anthropic::Client::from_env()?
                .agent(&model)
                .preamble(preamble)
                .build(),
        ),
        Provider::Cohere => Box::new(
            cohere::Client::from_env()?
                .agent(&model)
                .preamble(preamble)
                .build(),
        ),
        Provider::Ollama => Box::new(
            ollama::Client::from_env()?
                .agent(&model)
                .preamble(preamble)
                .build(),
        ),
        Provider::OpenRouter => Box::new(
            openrouter::Client::from_env()?
                .agent(&model)
                .preamble(preamble)
                .build(),
        ),
    };

    Ok(agent)
}

pub fn build_model_listing_client(provider: Provider) -> Result<Box<dyn ListModels>, AiError> {
    let client: Box<dyn ListModels> = match provider {
        Provider::OpenAI => Box::new(openai::Client::from_env()?),
        Provider::Gemini => Box::new(gemini::Client::from_env()?),
        Provider::Anthropic => Box::new(anthropic::Client::from_env()?),
        Provider::Ollama => Box::new(ollama::Client::from_env()?),
        Provider::OpenRouter => Box::new(openrouter::Client::from_env()?),
        Provider::Cohere => {
            return Err(AiError::Other(
                "Cohere does not support model listing".to_string(),
            ));
        }
    };

    Ok(client)
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumString, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Gemini,
    Anthropic,
    Cohere,
    Ollama,
    OpenRouter,
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("Rate limit exceeded {0}")]
    RateExceeded(String),
    #[error("Provider  does not provide model ")]
    NotFound,
    #[error("Provider client error: {0}")]
    ProviderError(String),
    #[error("Unknown error: {0}")]
    Other(String),
}
impl From<PromptError> for AiError {
    fn from(error: PromptError) -> Self {
        match error {
            PromptError::CompletionError(e) => match e {
                CompletionError::HttpError(e) => match e {
                    HttpError::InvalidStatusCode(s) => match s {
                        StatusCode::TOO_MANY_REQUESTS => AiError::RateExceeded(e.to_string()),
                        StatusCode::NOT_FOUND => AiError::NotFound,
                        _ => {
                            dbg!(&s, s.as_u16());
                            AiError::Other(e.to_string())
                        }
                    },
                    HttpError::InvalidStatusCodeWithMessage(code, msg) => match code.as_u16() {
                        429 => AiError::RateExceeded(msg.to_string()),
                        404 => AiError::NotFound,
                        _ => AiError::Other(format!("{code}: {msg}")),
                    },
                    _ => {
                        dbg!(&e);
                        AiError::Other(e.to_string())
                    }
                },
                _ => AiError::Other(e.to_string()),
            },
            _ => AiError::Other(error.to_string()),
        }
    }
}

impl From<ProviderClientError> for AiError {
    fn from(e: ProviderClientError) -> Self {
        AiError::ProviderError(e.to_string())
    }
}

#[async_trait::async_trait]
pub trait ListModels: Send + Sync {
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>>;
}

#[async_trait::async_trait]
impl<T> ListModels for T
where
    T: ModelListingClient + Send + Sync,
{
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>> {
        Ok(ModelListingClient::list_models(self)
            .await?
            .into_iter()
            .map(|m| ModelEntry {
                display: format!("{} ({})", m.display_name(), m.id),
                id: m.id.to_string(),
            })
            .collect())
    }
}




const MOCK_RESPONSE:&str ="feat: add file test.txt";

pub struct MockAi {
    pub response: String,
}

impl Default for MockAi {
    fn default() -> Self {
        Self {
            response: MOCK_RESPONSE.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl GenerateCommitMsg for MockAi {
    async fn generate_commit_msg(&self, _diff: &str) -> Result<String, AiError> {
        Ok(self.response.clone())
    }
}

#[async_trait::async_trait]
impl ListModels for MockAi {
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>> {
        Ok(vec![ModelEntry {
            display: "mock-model (mock-1)".to_string(),
            id: "mock-1".to_string(),
        }])
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup;
    use crate::git::stage_files;

    #[tokio::test]
    async fn test_commit_msg_gen() -> anyhow::Result<()> {
        let (repository, _dir) = setup()?;
        stage_files(&["test.txt".to_string()], &repository)?;
        
        let diff = crate::git::get_diff(&repository)?.expect("diff should exist");
        let agent = MockAi::default();
        let msg = agent.generate_commit_msg(&diff).await?;
        
        assert_eq!(msg, MOCK_RESPONSE);
        Ok(())
    }
}