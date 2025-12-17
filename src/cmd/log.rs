use super::{
    args::{GlobalArgs, LogArgs},
    state::State,
};

use crate::utils::print::pretty_print_logs;

pub fn run(
    args: &LogArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(None)?;

    //let logs = get_logs(args.number, args.reverse)?;

    //pretty_print_logs(&logs, global.compact)?;

    Ok(())
}
