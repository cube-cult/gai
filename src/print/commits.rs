use crate::git::commit::GitCommit;

pub fn print(
    commits: &[GitCommit],
    compact: bool,
) -> anyhow::Result<Option<usize>> {
    let selected = None;

    Ok(selected)
}
