use crate::{
    args::{FindArgs, GlobalArgs},
    git::log::get_logs,
    print::query,
    state::State,
};

pub fn run(
    args: &FindArgs,
    _global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(None)?;

    let count = args
        .number
        .unwrap_or_default();

    get_logs(&state.git.repo, count, args.reverse)?;

    let q = query("What is your query?")?;

    println!("{q}");

    Ok(())
}
