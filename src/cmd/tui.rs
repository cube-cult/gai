use super::args::{GlobalArgs, TUIArgs};

pub fn run(
    _args: &TUIArgs,
    _global: &GlobalArgs,
) -> anyhow::Result<()> {
    // todo deprecate
    //let state = State::new(global.config.as_deref())?;
    //crate::tui::open(state.settings, state.gai)
    Ok(())
}
