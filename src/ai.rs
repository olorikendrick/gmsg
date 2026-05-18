use http::status::StatusCode;
use rig::{
    agent::{Agent, PromptHook},
    client::{CompletionClient, ProviderClient, ProviderClientError},
    completion::{CompletionError, CompletionModel, Prompt, PromptError},
    http_client::Error as HttpError,
    providers::{anthropic, cohere, gemini, ollama, openai, openrouter},
};

use serde::{Deserialize, Serialize};
use strum::EnumIter;
use strum_macros::{Display, EnumString};

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
