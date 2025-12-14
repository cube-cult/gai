use super::{
    args::{GlobalArgs, TUIArgs},
    state::State,
};

pub fn run(
    _args: &TUIArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(global.config.as_deref())?;

    crate::tui::open(state.settings, state.gai)
}
