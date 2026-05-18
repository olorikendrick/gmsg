use std::path::Path;

use anyhow::{self, Context};
use git2::{DiffFormat, DiffOptions, Repository, Status, StatusEntry, Tree};

pub fn get_diff(repository: &Repository) -> anyhow::Result<Option<String>> {
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
        Ok(Some(output))
    } else {
        Ok(None)
    }
}

pub fn commit(repository: &Repository, message: &str) -> anyhow::Result<()> {
    let signature = repository
        .signature()
        .context("Could not read repository Signature")?;
    let parent = match repository.head() {
        Ok(head) => head.peel_to_commit().ok(),
        Err(_) => None,
    };
    let mut index = repository.index().context("Could not get index")?;
    let tree_id = index.write_tree().context("Could not get tree of head")?;
    let tree = repository
        .find_tree(tree_id)
        .context("Could not find tree")?;

    let parents_slice: Vec<&git2::Commit> = parent.iter().collect();
    repository
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents_slice,
        )
        .context("Could not make commit")?;

    Ok(())
}

pub fn get_staged_files(repository: &Repository) -> anyhow::Result<Option<Vec<String>>> {
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

pub fn stage_files(paths: &[String], repository: &Repository) -> anyhow::Result<()> {
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
#[cfg(test)]
mod test {
    use crate::git::get_staged_files;
    use crate::git::{commit, get_diff, stage_files};
    use anyhow::{Context, Result};
    use git2::Repository;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};

    fn setup() -> Result<(Repository, TempDir)> {
        let directory = tempfile::tempdir()?;

        let dir = directory.path();
        let repository = Repository::init(&dir).context("Could not initialize repository")?;

        let mut config = repository.config()?;
        config.set_str("user.name", "test")?;
        config.set_str("user.email", "test@test.com")?;

        let file = "Test file";
        fs::write(dir.join("test.txt"), file)?;

        Ok((repository, directory))
    }
    #[test]
    fn test_stage_files_works() -> Result<()> {
        let (repository, dir) = setup()?;
        let result = stage_files(&vec!["test.txt".to_string()], &repository);
        assert!(result.is_ok());
        let diff = get_diff(&repository).context("Could not get diff")?;

        assert!(diff.is_some());

        Ok(())
    }

    #[test]
    fn test_commit_on_empty_repo_works() -> Result<()> {
        let (repository, dir) = setup()?;
        stage_files(&vec!["test.txt".to_string()], &repository)?;
        let diff = get_diff(&repository).context("Could not get diff")?;

        assert!(diff.is_some());

        let result = commit(&repository, "First commit");
        let diff = get_diff(&repository).context("Could not get diff")?;

        assert!(diff.is_none());

        assert!(result.is_ok());
        let _ = dir.path();

        Ok(())
    }

    #[test]
    fn test_get_staged_files_with_staged_files() -> Result<()> {
        let (repository, directory) = setup()?;
        let file = "test.txt".to_string();
        stage_files(&vec![file.clone()], &repository)?;
        let result = get_staged_files(&repository)?;
        assert!(result.is_some());
        let files = result.unwrap();
        assert_eq!(file, files[0]);

        Ok(())
    }
    #[test]
    fn test_get_staged_files_with_no_staged_files() -> Result<()> {
        let (repository, directory) = setup()?;

        let files = get_staged_files(&repository)?;
        assert!(files.is_none());

        Ok(())
    }
}
