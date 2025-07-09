use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct Config {
    /// Where we keep our vault, key, extensions, etc.
    pub base_dir: PathBuf,

    /// `<base_dir>/vault`
    pub prefix: PathBuf,

    /// `<base_dir>/private.agekey`
    pub secret: PathBuf,

    /// `<base_dir>/extensions`
    #[allow(dead_code)]
    pub extensions_dir: PathBuf,

    /// Clipboard timeout (seconds)
    #[allow(dead_code)]
    pub clip_time: u64,
}

pub fn load_config() -> Result<Config> {
    // 1) Honor explicit override
    let base_dir = if let Ok(dir) = env::var("PASSMAN_DIR") {
        PathBuf::from(dir)
    } else {
        // 2) Fallback to XDG_DATA_HOME or default `~/.local/share`
        let data_home = env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .expect("HOME directory not set")
                    .join(".local")
                    .join("share")
            });
        data_home.join("passman")
    };

    // sub-directories / files
    let prefix = base_dir.join("vault");
    let secret = base_dir.join("private.agekey");
    let extensions_dir = base_dir.join("extensions");

    // ensure structure
    fs::create_dir_all(&prefix).context("Failed to create passman vault directory")?;
    fs::create_dir_all(&extensions_dir).context("Failed to create passman extensions directory")?;

    // clip timeout override
    let clip_time = env::var("PASSWORD_STORE_CLIP_TIME")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(45);

    Ok(Config {
        base_dir,
        prefix,
        secret,
        extensions_dir,
        clip_time,
    })
}
