use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

pub mod gemini;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
}

#[async_trait]
pub trait CompletionClient: Send + Sync {
    async fn generate_commit_msg(&self, prompt: &str) -> anyhow::Result<String>;
    async fn prompt(&self, prompt: &str) -> anyhow::Result<String>;
}

#[async_trait]
pub trait ModelProvider {
    async fn list_models(&self) -> anyhow::Result<Vec<ModelEntry>>;

    fn into_completion_client(
        &self,
        model: ModelEntry,
        sys_prompt: String,
    ) -> anyhow::Result<Box<dyn CompletionClient>>;
}

#[derive(Clone, Serialize, Deserialize, Debug, EnumString, EnumIter, Display)]
pub enum Provider {
    Gemini,
}

impl Provider {
    pub fn initialize(&self) -> anyhow::Result<Box<dyn ModelProvider>> {
        match self {
            Provider::Gemini => {
                use gemini::GeminiProvider;
                Ok(Box::new(GeminiProvider::from_env()?))
            }
        }
    }
}
