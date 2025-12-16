use crate::git::{
    diffs::get_diffs, repo::GitRepo, settings::DiffStrategy,
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

    let diffs = get_diffs(&git_repo, &strategy)?;

    for diff in diffs.files {
        println!("{:#?}", diff.hunks);
    }

    cmd::run()
}
