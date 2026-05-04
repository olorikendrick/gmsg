use anyhow::Context;
use clap::Parser;
use git2::{DiffFormat, Repository, Status, StatusEntry};
use gmsg::types::Gmsg;
use std::io::Write;
use std::{fs::File, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let cli = Gmsg::parse();

    let filter_staged = |status: &StatusEntry| {
        status.status().intersects(
            Status::INDEX_DELETED
                | Status::INDEX_MODIFIED
                | Status::INDEX_NEW
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        )
    };

    let wdir: PathBuf = if let Some(path) = cli.path.as_ref() {
        eprintln!("Path supplied ,{:?}", &path);
        path.to_owned()
    } else {
        let dir = std::env::current_dir().context("Failed to open current working directory")?;
        eprintln!("No path supplied ,using current directory ,{:?}", &dir);
        dir
    };

    let repository =
        Repository::open(wdir).context("Failed to open a git repository in the specified directory,Check if it exists or if you have neccessary permisiions")?;
    let statuses = repository.statuses(None).unwrap();
    if statuses.iter().any(|s| filter_staged(&s)) {
        for status in statuses.iter().filter(filter_staged) {
            dbg!(status.path());
        }
    } else {
        println!("No staged Changes Detected");
        return Ok(());
    }
    

    Ok(())
}
