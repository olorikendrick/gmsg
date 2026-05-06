use rig::{
    agent::{Agent, PromptHook},
    client::{CompletionClient, ProviderClient},
    completion::{CompletionModel, Prompt},
    providers::gemini,
};

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

pub fn build_commit_agent(system_message: Option<&str>) -> anyhow::Result<impl GenerateCommitMsg> {
    let preamble = if let Some(msg) = system_message {
        msg
    } else {
        SYSTEM_PROMPT
    };
    let model = gemini::Client::from_env()?;
    let agent = model.agent("gemini-2.5-flash").preamble(preamble).build();
    Ok(agent)
}
