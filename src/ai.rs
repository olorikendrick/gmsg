use rig::{
    agent::{Agent, PromptHook},
    client::{CompletionClient, ProviderClient},
    completion::{CompletionModel, Prompt},
    providers::{anthropic, cohere, gemini, ollama, openai, openrouter},
};

use serde::{Deserialize, Serialize};
use strum::EnumIter;
use strum_macros::{Display, EnumString};

const SYSTEM_PROMPT: &str = r#"
You will be given a git diff. Your task is to generate a commit message that describes ONLY the changes shown in the diff hunks (lines beginning with + or -). 


Be precise. Describe what changed, not what exists around it.
For small, focused changes keep the body concise. 
Only expand into detail when the change is complex or touches multiple systems and verbosity is deemed neccessary.
You should follow conventional commit specifications 
"#;

#[async_trait::async_trait]
pub trait GenerateCommitMsg {
    async fn generate_commit_msg(&self, diff: &str) -> anyhow::Result<String>;
}

#[async_trait::async_trait]
impl<M, P> GenerateCommitMsg for Agent<M, P>
where
    M: CompletionModel + 'static,
    P: PromptHook<M> + 'static,
{
    async fn generate_commit_msg(&self, diff: &str) -> anyhow::Result<String> {
        Ok(self.prompt(diff).await?)
    }
}
pub fn build_commit_agent(
    provider: Provider,
    model: String,
    system_message: Option<&str>,
) -> anyhow::Result<Box<dyn GenerateCommitMsg>> {
    let preamble = system_message.unwrap_or(SYSTEM_PROMPT);

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
