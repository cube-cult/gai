use super::{
    args::{GlobalArgs, StatusArgs},
    state::State,
};
use crate::{
    git::{DiffStrategy, diffs::get_diffs},
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

        let mut diff_strategy = DiffStrategy {
            staged_only: state.settings.commit.only_staged,
            ..Default::default()
        };

        if let Some(ref files_to_truncate) =
            state.settings.context.truncate_files
        {
            diff_strategy.truncated_files =
                files_to_truncate.to_owned();
        }

        if let Some(ref files_to_ignore) =
            state.settings.context.ignore_files
        {
            diff_strategy.ignored_files = files_to_ignore.to_owned();
        }

        let diffs = get_diffs(&state.git, &diff_strategy)?;

        let req = build_request(
            &state.settings,
            &state.git,
            &diffs.to_string(),
        );

        spinner.stop(None);
        println!("{}", req);
    }

    Ok(())
}
