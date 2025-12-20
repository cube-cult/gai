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
        /* args::Commands::Rebase => {}
        args::Commands::Find => {}
        args::Commands::Bisect => {} */
    };

    Ok(())
}
