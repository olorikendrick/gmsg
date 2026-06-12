use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub mod gemini;
use strum_macros::{Display, EnumIter, EnumString};

#[async_trait]
pub trait GenerateCommitMsg {
    async fn generate_commit_msg(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
#[async_trait]
pub trait ListModels {
    async fn list_models(
        &self,
    ) -> Result<Vec<ModelEntry>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct ModelEntry {
    pub id: String,
    pub name: String,
}

pub trait CompletionProvider: GenerateCommitMsg + ListModels + Send + Sync {}

#[derive(Clone, Serialize, Deserialize, Debug, EnumString, EnumIter, Display)]
pub enum Provider {
    Gemini,
}
