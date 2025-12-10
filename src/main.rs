use clap::Parser;

pub mod args;
pub mod cmd;
pub mod configuration;
pub mod git;
pub mod providers;
pub mod state;
pub mod tui;
pub mod utils;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = args::Cli::parse();
    cmd::run(&args)?;

    Ok(())
}
