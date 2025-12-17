use super::{
    args::{GlobalArgs, StatusArgs},
    state::State,
};
use crate::{
    providers::request::build_request,
    utils::print::{SpinDeez, pretty_print_status},
};

pub fn run(
    args: &StatusArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(global.config.as_deref())?;

    pretty_print_status(&state.git, global.compact)?;

    if args.verbose {
        let spinner = SpinDeez::new();

        let req =
            build_request(&state.settings, &state.git, &spinner);
        println!("{}", req);
    }

    Ok(())
}
