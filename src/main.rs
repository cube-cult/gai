pub mod cmd;
pub mod git;
pub mod providers;
pub mod settings;
pub mod tui;
pub mod utils;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    cmd::run()
}
