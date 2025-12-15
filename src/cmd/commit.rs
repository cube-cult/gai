use dialoguer::{Confirm, Select, theme::ColorfulTheme};

use super::{
    args::{CommitArgs, GlobalArgs},
    state::State,
};
use crate::{
    git::{commit::GaiCommit, repo::GaiGit},
    providers::{provider::extract_from_provider, request::Request},
    settings::Settings,
    //tui::app::open,
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
    if args.hunks {
        state.settings.commit.stage_hunks = true;
    }
    if args.files {
        state.settings.commit.stage_hunks = false;
    }

    if let Some(provider) = global.provider {
        state.settings.provider = provider;
    }

    /* state.gai.create_diffs(
        state.settings.context.truncate_files.as_deref(),
    )?; */

    pretty_print_status(&state.gai, global.compact)?;

    /* if state.gai.files.is_empty() {
        return Ok(());
    } */

    let spinner = SpinDeez::new();

    let req = crate::providers::request::build_request(
        &state.settings,
        &state.gai,
        &spinner,
    );

    run_commit(
        &spinner,
        req,
        state.settings,
        state.gai,
        args.skip_confirmation,
        global.compact,
    )?;

    Ok(())
}

fn run_commit(
    spinner: &SpinDeez,
    req: Request,
    cfg: Settings,
    gai: GaiGit,
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

        pretty_print_commits(&result.commits, &cfg, &gai, compact)?;

        let commits: Vec<GaiCommit> = result
            .commits
            .iter()
            .map(|resp_commit| {
                GaiCommit::from_response(
                    resp_commit,
                    cfg.commit.capitalize_prefix,
                    cfg.commit.include_scope,
                )
            })
            .collect();

        if skip_confirmation {
            println!("Skipping confirmation and applying commits...");
            match gai.apply_commits(&commits) {
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
            match gai.apply_commits(&commits) {
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
            //let _ = open(cfg, gai);
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
