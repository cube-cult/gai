use super::{
    args::{GlobalArgs, TUIArgs},
    state::State,
};
use crate::tui::app::run_tui;

pub fn run(
    _args: &TUIArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(global.config.as_deref())?;

    run_tui(state.config, state.gai)?;

    Ok(())
}
