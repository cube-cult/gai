use super::{
    args::{GlobalArgs, StatusArgs},
    state::State,
};
use crate::{
    git::{DiffStrategy, diffs::get_diffs, status::get_status},
    providers::request::build_request,
    utils::print::SpinDeez,
};

pub fn run(
    args: &StatusArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(global.config.as_deref())?;

    let status_strategy = crate::git::StatusStrategy::default();

    let status = get_status(&state.git.repo, &status_strategy)?;
    println!("{}", status);

    //pretty_print_status(&state.git, global.compact)?;

    if args.verbose {
        let spinner = SpinDeez::new();

        let mut diff_strategy = DiffStrategy {
            status_strategy,
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
