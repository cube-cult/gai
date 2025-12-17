use crate::{
    git::repo::GitRepo,
    settings::{Settings, load},
};

pub struct State {
    pub settings: Settings,
    pub git: GitRepo,
}

impl State {
    pub fn new(overrides: Option<&[String]>) -> anyhow::Result<Self> {
        let settings = load::load(overrides)?;
        let git = GitRepo::open(None)?;

        Ok(Self { settings, git })
    }
}
