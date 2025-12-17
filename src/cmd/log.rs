use crate::git::log::get_logs;

use super::{
    args::{GlobalArgs, LogArgs},
    state::State,
};

pub fn run(
    args: &LogArgs,
    _global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(None)?;

    let count = args.number.unwrap_or_default();

    let logs = get_logs(&state.git.repo, count, args.reverse)?;

    println!("{}", logs);

    Ok(())
}
