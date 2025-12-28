pub mod cmd;
pub mod git;
pub mod print;
pub mod providers;
pub mod settings;
pub mod utils;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    cmd::run()
}
