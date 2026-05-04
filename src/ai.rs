use rig::{
    agent::{Agent, PromptHook},
    completion::{CompletionModel, Prompt},
};

pub trait GenerateCommitMsg {
    async fn generate_commit_msg(&self, diff: &str) -> anyhow::Result<String>;
}

impl<M, P> GenerateCommitMsg for Agent<M, P>
where
    M: CompletionModel + 'static,
    P: PromptHook<M> + 'static,
{
    async fn generate_commit_msg(&self, diff: &str) -> anyhow::Result<String> {
        Ok(self.prompt(diff).await?)
    }
}
