use crate::git::status::GitStatus;

use super::terminal;

pub fn print(
    status: &GitStatus,
    compact: bool,
) -> anyhow::Result<()> {
    let terminal = terminal::start()?;

    terminal::stop()?;

    Ok(())
}
