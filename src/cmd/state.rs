use crate::{
    git::{Diffs, GitRepo},
    settings::{Settings, load},
};

pub struct State {
    pub settings: Settings,
    pub git: GitRepo,

    /// diff database
    /// we'll use this to compare difflines
    /// over the next commits
    /// during hunk staging
    /// otherwise, diffs
    /// in these will get removed
    /// as we apply them
    pub diffs: Diffs,
}

impl State {
    pub fn new(overrides: Option<&[String]>) -> anyhow::Result<Self> {
        let settings = load::load(overrides)?;
        let git = GitRepo::open(None)?;
        let diffs = Diffs::default();

        Ok(Self {
            settings,
            git,
            diffs,
        })
    }
}
