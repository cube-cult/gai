pub mod cmd;
pub mod configuration;
pub mod git;
pub mod providers;
pub mod tui;
pub mod utils;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    cmd::run()
}
