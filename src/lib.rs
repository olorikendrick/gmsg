pub mod types;

pub mod ai;
pub mod editor;
pub mod git;

use crate::ai::GenerateCommitMsg;
use crate::editor::Editor;
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

    let diff = git::get_diff(&repository)?;

    let model = gemini::Client::from_env()?;
    let agent = model
        .agent("gemini-2.5-flash")
        .preamble(
            "You are a git expert. Write a conventional commit message based on the following diff.
Focus on what the change DOES from a user or system behavior perspective, not how the code changed internally.
Use the format: <type>(<scope>): <short description>\n\n<body>
The body should explain WHY the change was made, not WHAT changed in the code.
Be concise. Output only the commit message, nothing else.")
        .build();
    out = strip_backtick(&agent.generate_commit_msg(&diff).await?);
    if cli.interactive {
        let mut terminal = ratatui::init();
         out = Editor::from(out)
            .run(&mut terminal)
            .context("Failed to initialize inline editor")?;
        ratatui::restore();
    }
    match git::commit(&repository,&out) {
        Ok(_)=>{
            eprintln!("Committed wih message: \n{}",out);
        }
        Err(e)=>{
            eprintln!("Encountered Error While commiting {:?}",e);
        }
        
    }
    

    Ok(())
}

fn strip_backtick(input: &str) -> String {
    input.replace('`', "")
}
