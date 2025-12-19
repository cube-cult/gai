use std::collections::HashMap;

use dialoguer::{Confirm, Select, theme::ColorfulTheme};

use super::{
    args::{CommitArgs, GlobalArgs},
    state::State,
};
use crate::{
    git::{
        Diffs, GitRepo,
        commit::{GitCommit, commit},
        diffs::{FileDiff, HunkId, find_file_hunks, get_diffs},
        settings::{DiffStrategy, StagingStrategy},
        staging::{stage_file, stage_hunks},
    },
    providers::{
        provider::extract_from_provider,
        request::{Request, build_request},
    },
    settings::Settings,
    utils::print::{
        SpinDeez, pretty_print_commits, pretty_print_status,
    },
};

pub fn run(
    args: &CommitArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let mut state = State::new(global.config.as_deref())?;

    state.settings.prompt.hint = global.hint.to_owned();

    if args.staged {
        state.settings.commit.only_staged = true;
    }

    if let Some(provider) = global.provider {
        state.settings.provider = provider;
    }

    /* state.git.create_diffs(
        state.settings.context.truncate_files.as_deref(),
    )?; */

    pretty_print_status(&state.git, global.compact)?;

    /* if state.git.files.is_empty() {
        return Ok(());
    } */

    let spinner = SpinDeez::new();
    spinner.start("Building Request");

    let mut diff_strategy = DiffStrategy {
        staged_only: state.settings.commit.only_staged,
        ..Default::default()
    };

    if let Some(ref files_to_truncate) =
        state.settings.context.truncate_files
    {
        diff_strategy.truncated_files = files_to_truncate.to_owned();
    }

    if let Some(ref files_to_ignore) =
        state.settings.context.ignore_files
    {
        diff_strategy.ignored_files = files_to_ignore.to_owned();
    }

    state.diffs = get_diffs(&state.git, &diff_strategy)?;

    let req = build_request(
        &state.settings,
        &state.git,
        &state.diffs.to_string(),
    );

    spinner.stop(None);

    run_commit(
        &spinner,
        req,
        state.settings,
        state.git,
        state.diffs,
        args.skip_confirmation,
        global.compact,
    )?;

    Ok(())
}

fn run_commit(
    spinner: &SpinDeez,
    req: Request,
    cfg: Settings,
    git: GitRepo,
    mut diffs: Diffs,
    skip_confirmation: bool,
    compact: bool,
) -> anyhow::Result<()> {
    loop {
        spinner.start(&format!(
            "Awaiting response from {} using {}",
            &cfg.provider.to_string(),
            cfg.providers.get_model(&cfg.provider)
        ));

        let response = extract_from_provider(
            &cfg.provider,
            &req.prompt,
            &req.diffs,
        );

        let result = match response {
            Ok(r) => r,
            Err(e) => {
                spinner.stop(Some(
                    "Done! But Gai received an error from the provider:"
                ));

                println!("{:#}", e);

                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Retry?")
                    .interact()
                    .unwrap()
                {
                    continue;
                } else {
                    break;
                }
            }
        };

        if result.commits.is_empty() {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("No commits found... retry?")
                .interact()
                .unwrap()
            {
                continue;
            } else {
                break;
            }
        }

        spinner.stop(None);

        println!(
            "Done! Received {} Commit{}",
            result.commits.len(),
            if result.commits.len() == 1 { "" } else { "s" }
        );

        pretty_print_commits(&result.commits, &cfg, &git, compact)?;

        let git_commits: Vec<GitCommit> = result
            .commits
            .iter()
            .map(|resp_commit| resp_commit.into())
            .collect();

        if skip_confirmation {
            match apply_commits(&git, &git_commits, &mut diffs.files)
            {
                Ok(_) => break,
                Err(e) => {
                    println!("Failed to Apply Commits: {}", e);

                    let options = ["Retry", "Exit"];
                    let selection =
                        Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select an option:")
                            .items(options)
                            .default(0)
                            .interact()
                            .unwrap();

                    if selection == 0 {
                        println!("Retrying...");
                        continue;
                    } else if selection == 1 {
                        println!("Exiting");
                        break;
                    }
                }
            };
        }

        let options = ["Apply All", "Show in TUI", "Retry", "Exit"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an option:")
            .items(options)
            .default(0)
            .interact()
            .unwrap();

        if selection == 0 {
            println!("Applying Commits...");
            match apply_commits(&git, &git_commits, &mut diffs.files)
            {
                Ok(_) => break,
                Err(e) => {
                    println!("Failed to Apply Commits: {}", e);

                    let options = ["Retry", "Exit"];
                    let selection =
                        Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Select an option:")
                            .items(options)
                            .default(0)
                            .interact()
                            .unwrap();

                    if selection == 0 {
                        println!("Retrying...");
                        continue;
                    } else if selection == 1 {
                        println!("Exiting");
                        break;
                    }
                }
            }
        } else if selection == 1 {
            //let _ = open(cfg, git);
        } else if selection == 2 {
            println!("Retrying...");
            continue;
        } else if selection == 3 {
            println!("Exiting");
        }

        break;
    }

    Ok(())
}

fn apply_commits(
    git: &GitRepo,
    git_commits: &[GitCommit],
) -> anyhow::Result<()> {
    let staging_stragey = StagingStrategy::default();
    for git_commit in git_commits {
        if let StagingStrategy::AtomicCommits = staging_stragey {
            for file in &git_commit.files {
                stage_file(&git.repo, file)?;
            }
        }
        commit(&git.repo, git_commit)?;
    }

    Ok(())
}
