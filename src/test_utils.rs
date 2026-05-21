use anyhow::{Context, Result};
use git2::Repository;
use std::fs;
use tempfile::TempDir;

pub fn setup() -> Result<(Repository, TempDir)> {
    let directory = tempfile::tempdir()?;
    let dir = directory.path();
    let repository = Repository::init(dir).context("Could not initialize repository")?;

    let mut config = repository.config()?;
    config.set_str("user.name", "test")?;
    config.set_str("user.email", "test@test.com")?;

    fs::write(dir.join("test.txt"), "Test file")?;
    fs::write(
        dir.join(".gmsgconfig.toml"),
        "[ai]\n provider = \"mockai\"\n model = \"mock-1\"\n prompt=\"hey\"",
    )?;

    Ok((repository, directory))
}
