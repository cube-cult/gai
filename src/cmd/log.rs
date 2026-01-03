use crate::{git::log::get_logs, print::log};

use super::{
    args::{GlobalArgs, LogArgs},
    state::State,
};

pub fn run(
    args: &LogArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(None)?;

    let count = args.number.unwrap_or_default();

    let logs = get_logs(&state.git.repo, count, args.reverse)?;

    log::print(&logs.git_logs, global.compact, args.interactive)?;

    Ok(())
}
