use clap::Parser;

pub mod args;
pub mod cmds;
pub mod configuration;
pub mod git;
pub mod providers;
pub mod state;
pub mod terminal;
pub mod utils;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = args::Cli::parse();
    cmds::run(&args)?;

    Ok(())
}
