use anyhow::{Context};
use clap::Parser;
use git2::{DiffFormat, Repository, Status, StatusEntry, Tree};
use gmsg::git;
use gmsg::types::Gmsg;
use std::io::Write;
use std::{fs::File, path::PathBuf};

fn main() -> anyhow::Result<()> {
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
    git::run(repository)?;
   
    Ok(())
}
