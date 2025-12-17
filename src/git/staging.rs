// https://github.com/gitui-org/gitui/blob/master/asyncgit/src/sync/staging/mod.rs

// ripped from asyncgit
// necessary for staging math
// since we're getting multiple
// commits at a time
// we're gonna need to have to stage
// individual hunks as specified
// in the response separately
// depending on the accuracy,
// will make these operations separate

use git2::{IndexAddOption, Repository};
use std::path::Path;

/// for atomic commits
pub fn stage_file(
    repo: &Repository,
    path: &str,
) -> anyhow::Result<()> {
    let mut index = repo.index()?;

    index.add_path(Path::new(path))?;
    index.write()?;

    Ok(())
}

/// for allfilesonecommit
pub fn stage_all(
    repo: &Repository,
    pattern: &str,
) -> anyhow::Result<()> {
    let mut index = repo.index()?;

    index.add_all(vec![pattern], IndexAddOption::DEFAULT, None)?;

    index.write()?;

    Ok(())
}
