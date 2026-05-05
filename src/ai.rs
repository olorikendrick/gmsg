use rig::{
    agent::{Agent, PromptHook},
    client::{CompletionClient, ProviderClient},
    completion::{CompletionModel, Prompt},
    providers::gemini,
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

pub fn build_commit_agent() -> anyhow::Result<impl GenerateCommitMsg> {
    let model = gemini::Client::from_env()?;
    let agent =     model
        .agent("gemini-2.5-flash")
        .preamble(
            "You are a git expert. Write a conventional commit message based on the following diff.
Focus on what the change DOES from a user or system behavior perspective, not how the code changed internally.
Use the format: <type>(<scope>): <short description>\n\n<body>
The body should explain WHY the change was made, not WHAT changed in the code.
Be concise. Output only the commit message, nothing else.
Carefully read the entire diff. If multiple distinct changes are present, describe all of them in the commit body — do not omit smaller changes like documentation, comments, or formatting.
use this guide:
https://www.conventionalcommits.org/en/v1.0.0/
")
        .build();
    Ok(agent)
}
