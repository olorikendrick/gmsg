use std::path::Path;

use anyhow::{self, Context};
use git2::{DiffFormat, DiffOptions, Repository, Status, StatusEntry, Tree};

pub fn get_diff(repository: &Repository) -> anyhow::Result<String> {
    if get_staged_files(repository)
        .context("Could not get staged files")?
        .is_some()
    {
        let tree: Option<Tree> = match repository.head() {
            Ok(head) => head.peel_to_tree().ok(),
            Err(_) => None,
        };
        let index = repository.index().ok();
        let mut options = DiffOptions::new();

        options
            .ignore_whitespace_eol(true)
            .ignore_blank_lines(true)
            .context_lines(10);
        let diff = repository
            .diff_tree_to_index(tree.as_ref(), index.as_ref(), Some(&mut options))
            .context("Could not get diff")?;
        let mut output = String::new();
        let mut error: Option<anyhow::Error> = None;
        diff.print(DiffFormat::Patch, |_, _, line| {
            match str::from_utf8(line.content()) {
                Ok(s) => {
                    output.push_str(s);
                    true
                }
                Err(e) => {
                    error = Some(anyhow::anyhow!(e));
                    false
                }
            }
        })
        .context("Failed to print diff")?;
        if let Some(e) = error {
            return Err(e);
        }
        Ok(output)
    } else {
        Ok("No staged files detected".to_string())
    }
}

pub fn commit(repository: &Repository, message: &str) -> anyhow::Result<()> {
    let signature = repository
        .signature()
        .context("Could not read repository Signature")?;
    let head = repository.head().context("Could not get repository head")?;
    let mut index = repository.index().context("Could not get index")?;
    let tree_id = index.write_tree().context("Could not get tree of head")?;
    let tree = repository
        .find_tree(tree_id)
        .context("Could not find tree")?;
    let parents = head
        .peel_to_commit()
        .context("Could not get Parent of last commit")?;
    repository
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parents],
        )
        .context("Could not make commit")?;

    Ok(())
}

fn get_staged_files(repository: &Repository) -> anyhow::Result<Option<Vec<String>>> {
    let filter_staged = |status: &StatusEntry| {
        status.status().intersects(
            Status::INDEX_DELETED
                | Status::INDEX_MODIFIED
                | Status::INDEX_NEW
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        )
    };

    let statuses = repository
        .statuses(None)
        .context("Could not get status of current repo")?;

    let files: Result<Vec<String>, anyhow::Error> = statuses
        .iter()
        .filter(|s| filter_staged(s))
        .map(|s| {
            s.path()
                .ok_or_else(|| anyhow::anyhow!("Path Contains Invalid UTF-8"))
                .map(|p| p.to_owned())
        })
        .collect();
    let files = files?;
    if files.is_empty() {
        Ok(None)
    } else {
        Ok(Some(files))
    }
}

fn stage_files(paths: &[String], repository: &Repository) -> anyhow::Result<()> {
    if paths.is_empty() {
        return Err(anyhow::anyhow!("No path"));
    }
    let mut index = repository.index()?;
    for path in paths {
        let path = Path::new(&path);
        index.add_path(path)?;
    }
    index.write()?;
    Ok(())
}
