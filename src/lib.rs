pub mod types;

pub mod ai;
pub mod git;

use crate::ai::GenerateCommitMsg;
use anyhow::Context;
use clap::Parser;
use git2::Repository;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::gemini;
use std::path::PathBuf;

use crate::types::Gmsg;

pub async fn run() -> anyhow::Result<()> {
    let cli = Gmsg::parse();
    let mut out = String::new();

    let wdir: PathBuf = if let Some(path) = cli.path.as_ref() {
        eprintln!("Path supplied ,{:?}", &path);
        path.to_owned()
    } else {
        let dir = std::env::current_dir().context("Failed to open current working directory")?;
        eprintln!("No path supplied ,using current directory ,{:?}", &dir);
        dir
    };

    let repository =
        Repository::open(wdir)
        .context("Failed to open a git repository in the specified directory,Check if it exists or if you have neccessary permisions")?;

    let diff = git::run(repository)?;

    let model = gemini::Client::from_env()?;
    let agent = model
        .agent("gemini-2.5-flash")
        .preamble("You are a git expert,Write a standard commit message based on this diffs using conventional commit")
        .build();
    out = agent.generate_commit_msg(&diff).await?;
    println!("{}", out);

    Ok(())
}
