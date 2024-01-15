use std::path::Path;

use anyhow::bail;
use git2::{Repository, Signature};

pub fn init<P: AsRef<Path>>(path: P) -> anyhow::Result<Repository> {
    Ok(git2::Repository::init(path)?)
}

pub fn add_all(repo: &Repository, files: &[&str]) -> anyhow::Result<()> {
    let mut index = repo.index()?;
    index.add_all(files, git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

/// Commits the files in the index
///
/// Fails if the index is empty
pub fn commit(repo: &Repository, msg: &str) -> anyhow::Result<()> {
    // Source: Example of how to do init https://github.com/rust-lang/git2-rs/blob/fd4d7c7c840788ccdc535889a42532c3d57d338d/examples/init.rs#L94-L95
    // Second Source: Needed to see how to find parent https://github.dev/rust-lang/git2-rs/blob/master/examples/add.rs

    let sig = Signature::now("test_user", "test_email")?;
    let mut index = repo.index()?;
    if index.is_empty() {
        bail!("Empty index found. Unable to commit");
    }
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Get head to set as parent if applicable
    if let Ok(head) = repo.head() {
        if let Some(parent) = head.target() {
            let parent_commit = repo.find_commit(parent)?;
            repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[&parent_commit])?;
            return Ok(()); // Completed end here
        }
    }

    // Assumes there are no commits to use as parent
    repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[])?;
    Ok(())
}

pub fn commit_irrelevant_msg(repo: &Repository) -> anyhow::Result<()> {
    commit(repo, "no msg set")
}
