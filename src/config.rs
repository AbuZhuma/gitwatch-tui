use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    #[serde(default)]
    pub repos: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    #[serde(default)]
    groups: Vec<Group>,
}

pub fn load() -> Vec<Group> {
    let Some(path) = config_path() else {
        return Vec::new();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    toml::from_str::<Config>(&text)
        .map(|config| config.groups)
        .unwrap_or_default()
}

pub fn save(groups: &[Group]) -> Result<()> {
    let path = config_path().context("could not determine the config directory")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("could not create the config directory")?;
    }

    let config = Config {
        groups: groups.to_vec(),
    };
    let text = toml::to_string_pretty(&config).context("could not serialize config")?;
    fs::write(&path, text).context("could not write the config file")?;
    Ok(())
}

fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "gitwatch")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}
