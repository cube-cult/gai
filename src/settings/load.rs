use config::{Config, ConfigError, File};
use directories::ProjectDirs;

use super::Settings;

pub fn load(
    overrides: Option<&[String]>,
) -> anyhow::Result<Settings> {
    let mut builder = Config::builder();

    if let Some(cfg_path) =
        ProjectDirs::from("com", "nuttycream", "gai")
            .map(|d| d.config_dir().to_owned())
        && cfg_path.join("config.toml").exists()
    {
        builder =
            builder.add_source(File::from(cfg_path).required(false));
    }

    if let Some(overrides) = overrides {
        for override_str in overrides {
            if let Some((key, value)) = override_str.split_once('=') {
                builder = builder.set_override(key, value)?;
            }
        }
    }

    // sticking with this method, dont create
    // a config file if it doesn't exist
    // instead use the config file
    // as an additional override along with any
    // cli passed overrides

    let settings = match builder.build() {
        Ok(cfg) => cfg.try_deserialize().unwrap_or_default(),
        Err(ConfigError::NotFound(_)) => Settings::default(),
        Err(e) => return Err(e.into()),
    };

    Ok(settings)
}
