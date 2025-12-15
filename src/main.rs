use crate::git::{
    diffs::Diffs, repo::GitRepo, settings::DiffStrategy,
};

pub mod cmd;
pub mod git;
pub mod providers;
pub mod settings;
//pub mod tui;
pub mod utils;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let git_repo = GitRepo::open(None)?;
    let strategy = DiffStrategy::default();
    Diffs::create(&git_repo, &strategy)?;

    cmd::run()
}
