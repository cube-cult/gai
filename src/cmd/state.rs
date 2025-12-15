use crate::{
    git::repo::GaiGit,
    settings::{Settings, load},
};

pub struct State {
    pub settings: Settings,
    pub gai: GaiGit,
}

impl State {
    pub fn new(overrides: Option<&[String]>) -> anyhow::Result<Self> {
        let settings = load::load(overrides)?;

        let gai = GaiGit::new(
            settings.commit.only_staged,
            settings.commit.stage_hunks,
            settings.commit.capitalize_prefix,
            settings.commit.include_scope,
        );

        Ok(Self { settings, gai })
    }
}
