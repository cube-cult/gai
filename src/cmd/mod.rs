use clap::Parser;

pub mod args;
pub mod auth;
pub mod commit;
pub mod log;
pub mod state;
pub mod status;

pub fn run() -> anyhow::Result<()> {
    let args = args::Cli::parse();

    match &args.command {
        args::Commands::Auth(a) => auth::run(&a.auth)?,
        args::Commands::Status(a) => status::run(a, &args.global)?,
        args::Commands::Commit(a) => commit::run(a, &args.global)?,
        args::Commands::Log(a) => log::run(a, &args.global)?,
        args::Commands::Test(a) => test_cmd(a, &args.global)?,
        /* args::Commands::Rebase => {}
        args::Commands::Find => {}
        args::Commands::Bisect => {} */
    };

    Ok(())
}

fn test_cmd(
    args: &args::TestArgs,
    global: &args::GlobalArgs,
) -> anyhow::Result<()> {
    use crate::print::*;

    let count = args.count.unwrap_or(5);

    match args.command {
        args::TestCommands::Status => {
            let status = test_gen_status(count);
            status::print(
                &status.branch_name,
                &status.statuses,
                &status.statuses,
                global.compact,
            )?;
        }
        args::TestCommands::Commit => {
            let commits = test_gen_commits(count);
            //commits::print(&commits, global.compact)?;
        }
    }

    Ok(())
}

fn test_gen_status(count: usize) -> crate::git::status::GitStatus {
    use crate::git::status::{FileStatus, GitStatus, StatusItemType};

    let statuses = (1..=count)
        .map(|i| {
            let status = match i % 4 {
                0 => StatusItemType::New,
                1 => StatusItemType::Modified,
                2 => StatusItemType::Deleted,
                _ => StatusItemType::Renamed,
            };

            FileStatus {
                path: format!("src/file{i}.rs"),
                status,
            }
        })
        .collect();

    GitStatus {
        branch_name: "main".to_string(),
        statuses,
    }
}

fn test_gen_commits(
    count: usize
) -> Vec<crate::git::commit::GitCommit> {
    use crate::git::commit::GitCommit;

    (1..=count)
        .map(|i| GitCommit {
            files: vec![format!("src/file{i}.rs")],
            hunk_ids: vec![format!("hunk{i}")],
            message: format!(
                "feat: commit {i}\n\nBody for commit {i}."
            ),
        })
        .collect()
}
