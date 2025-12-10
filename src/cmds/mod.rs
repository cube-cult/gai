use crate::args::{Cli, Commands};

pub mod auth;
pub mod commit;
pub mod log;
pub mod status;
pub mod tui;

pub fn run(args: &Cli) -> anyhow::Result<()> {
    match &args.command {
        Commands::Auth(a) => auth::run(&a.auth)?,
        Commands::Status(a) => status::run(a, &args.global)?,
        Commands::TUI(a) => tui::run(a)?,
        Commands::Commit(a) => commit::run(a, &args.global)?,
        Commands::Log(a) => log::run(a, &args.global)?,
        Commands::Rebase => {}
        Commands::Find => {}
        Commands::Bisect => {}
    };

    Ok(())
}
