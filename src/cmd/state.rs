use crate::{git::repo::GaiGit, settings::Settings};

pub struct State {
    pub config: Settings,
    pub gai: GaiGit,
}

impl State {
    pub fn new(overrides: Option<&[String]>) -> anyhow::Result<Self> {
        let config = Settings::load(overrides)?;

        let gai = GaiGit::new(
            config.gai.only_staged,
            config.gai.stage_hunks,
            config.gai.commit_config.capitalize_prefix,
            config.gai.commit_config.include_scope,
        )?;

        Ok(Self { config, gai })
    }
}
