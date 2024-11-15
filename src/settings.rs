use std::{fs, path::Path};

use anyhow::Context;
use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize,Default)]
pub struct Settings {
    #[serde(default)]
    pub github_token: String,
    #[serde(default)]
    pub github_pusher_repo: String,
    #[serde(default)]
    pub ak:String,
    #[serde(default)]
    pub sk:String,
}

pub fn load_config(path: &Path) -> anyhow::Result<Settings> {
    
    let s = Config::builder()
    .add_source(config::Environment::default().try_parsing(true))
    .add_source(config::File::from(path).required(false))
    .build()?;

    Ok(s.try_deserialize()?)
}

pub fn save_config(path: &Path, settings:Settings) -> anyhow::Result<()> {
    let yaml = toml::to_string(&settings)?;
    fs::write(path, yaml)?;
    println!("Config saved to {}", path.display());
    Ok(())
}