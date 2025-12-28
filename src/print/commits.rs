use crate::git::commit::GitCommit;

use super::terminal;

pub fn print(
    commits: &[GitCommit],
    compact: bool,
) -> anyhow::Result<Option<usize>> {
    let selected = None;

    let terminal = terminal::start(0)?;

    terminal::stop()?;

    Ok(selected)
}
