pub mod gmsg;

pub mod ai;
pub mod editor;
pub mod git;

use crate::ai::GenerateCommitMsg;
use crate::editor::Editor;
use anyhow::Context;
use clap::Parser;
use git2::Repository;
use std::path::PathBuf;

use crate::gmsg::Gmsg;

pub async fn run() -> anyhow::Result<()> {
    let cli = Gmsg::parse();

    let wdir: PathBuf = if let Some(path) = cli.path.as_ref() {
        eprintln!("Path supplied ,{:?}", &path);
        path.to_owned()
    } else {
        let dir = std::env::current_dir().context("Failed to open current working directory")?;
        eprintln!("No path supplied ,using current directory ,{:?}", &dir);
        dir
    };

    let repository =
        Repository::discover(wdir)
        .context("Failed to open a git repository in the specified directory,Check if it exists or if you have neccessary permisions")?;

    let diff = git::get_diff(&repository)?;
    let agent = ai::build_commit_agent(None).context("Could not Bootstrap Agent")?;
   let mut  out = strip_backtick(&agent.generate_commit_msg(&diff).await?);
    if cli.interactive {
        let mut terminal = ratatui::init();
        out = Editor::from(out)
            .run(&mut terminal)
            .context("Failed to initialize inline editor")?;
        ratatui::restore();
    }
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        println!("{}", out);
        return Ok(());
    }
    if cli.copy {
        //copy
        return Ok(());
    }
    match git::commit(&repository, &out) {
        Ok(_) => {
            eprintln!("Committed wih message: \n{}", out);
        }
        Err(e) => {
            eprintln!("Encountered Error While commiting {:?}", e);
        }
    }

    Ok(())
}

fn strip_backtick(input: &str) -> String {
    input.replace('`', "")
}
