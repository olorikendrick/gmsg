use anyhow::Context;
use git2::{DiffFormat, Repository, Status, StatusEntry, Tree};

pub fn get_diff(repository: &Repository) -> anyhow::Result<String> {
    let filter_staged = |status: &StatusEntry| {
        status.status().intersects(
            Status::INDEX_DELETED
                | Status::INDEX_MODIFIED
                | Status::INDEX_NEW
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        )
    };

    let mut out = String::new();
    let statuses = repository.statuses(None).unwrap();
    if statuses.iter().any(|s| filter_staged(&s)) {
        for status in statuses.iter().filter(filter_staged) {
            dbg!(status.path());
        }
    } else {
        println!("No staged Changes Detected");
        std::process::exit(0);
    }

    let tree: Option<Tree> = match repository.head() {
        Ok(head) => match head.peel_to_tree() {
            Ok(tree) => Some(tree),
            Err(_) => None,
        },
        Err(_) => None,
    };
    let index = match repository.index() {
        Ok(index) => Some(index),
        Err(_) => None,
    };
    let diff = repository
        .diff_tree_to_index(tree.as_ref(), index.as_ref(), None)
        .context("Could not get diff")?;
    diff.print(DiffFormat::Patch, |_, _, line| {
        out.push_str(
            str::from_utf8(line.content())
                .context("Could not parse diff")
                .unwrap(),
        );
        true
    })
    .unwrap();
    Ok(out)
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
    let parents = head.peel_to_commit().unwrap();
    repository
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parents],
        )
        .unwrap();

    Ok(())
}
