use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub base_dir: PathBuf,
    pub prefix: PathBuf,
    pub identities_file: PathBuf,
    pub extensions_dir: PathBuf,
    pub clip_time: u64,
}

pub fn load_config() -> Result<Config> {
    let base_dir = env::var("PASSMAN_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".passman"));

    let prefix = base_dir.join("store");
    let identities_file = base_dir.join("identities");
    let extensions_dir = base_dir.join("extensions");

    // Ensure directory structure exists
    fs::create_dir_all(&prefix).context("Failed to create store directory")?;
    fs::create_dir_all(&extensions_dir).context("Failed to create extensions directory")?;

    Ok(Config {
        base_dir,
        prefix,
        identities_file,
        extensions_dir,
        clip_time: env::var("PASSWORD_STORE_CLIP_TIME")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(45),
    })
}
