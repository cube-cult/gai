pub mod ai;
pub mod args;
pub mod auth;
pub mod config;
pub mod consts;
pub mod git;
pub mod graph;
pub mod print;
pub mod tui;

use anyhow::Result;
use clap::Parser;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use dotenv::dotenv;

use crate::{
    ai::{provider::extract_from_provider, request::Request},
    args::{Args, Auth, Commands},
    auth::{auth_login, auth_status, clear_auth},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    print::{SpinDeez, pretty_print_commits, pretty_print_status},
    tui::app::run_tui,
};

fn main() -> Result<()> {
    dotenv().ok();
    let mut cfg = config::Config::init()?;

    let args = Args::parse();
    let spinner = SpinDeez::new()?;

    args.parse_flags(&mut cfg)?;

    match args.command {
        Commands::Auth { ref auth } => {
            run_auth(auth, &spinner)?;
        }

        _ => {
            let mut gai = GaiGit::new(
                cfg.gai.only_staged,
                cfg.gai.stage_hunks,
                cfg.gai.commit_config.capitalize_prefix,
                cfg.gai.commit_config.include_scope,
            )?;

            gai.create_diffs(&cfg.ai.files_to_truncate)?;

            if gai.files.is_empty() {
                pretty_print_status(&gai, args.compact)?;
                return Ok(());
            }

            pretty_print_status(&gai, args.compact)?;

            match args.command {
                Commands::TUI { .. } => {
                    run_tui(cfg, gai)?;
                }
                Commands::Commit {
                    skip_confirmation,
                    config,
                    ..
                } => {
                    let cfg = match config {
                        Some(c) => cfg.override_cfg(&c)?,
                        None => cfg,
                    };

                    let req = build_request(&cfg, &gai, &spinner);

                    run_commit(
                        &spinner,
                        req,
                        cfg,
                        gai,
                        skip_confirmation,
                        args.compact,
                    )?;
                }
                Commands::Status { verbose } => {
                    if verbose {
                        let req = build_request(&cfg, &gai, &spinner);
                        println!("{}", req);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn build_request(
    cfg: &Config,
    gai: &GaiGit,
    spinner: &SpinDeez,
) -> Request {
    spinner.start("Building Request...");
    let mut req = Request::default();
    req.build_prompt(cfg, gai);
    req.build_diffs_string(gai.get_file_diffs_as_str());
    spinner.stop(None);
    req
}

fn run_auth(auth: &Auth, spinner: &SpinDeez) -> Result<()> {
    match auth {
        Auth::Login => auth_login()?,
        Auth::Status => auth_status(spinner)?,
        Auth::Logout => clear_auth()?,
    }

    Ok(())
}

fn run_commit(
    spinner: &SpinDeez,
    req: Request,
    cfg: Config,
    gai: GaiGit,
    skip_confirmation: bool,
    compact: bool,
) -> Result<()> {
    loop {
        spinner.start(&format!(
            "Awaiting response from {} using {}",
            cfg.ai.provider, "todo!"
        ));

        let response = extract_from_provider(
            &cfg.ai.provider,
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
                    cfg.gai.commit_config.capitalize_prefix,
                    cfg.gai.commit_config.include_scope,
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
            let _ = run_tui(cfg, gai);
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
