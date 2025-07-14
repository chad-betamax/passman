// src/utils/gather_config.rs

use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::{collections::HashMap, env};
use which::which;

use crate::config::{Config, load_config};

/// A serializable snapshot of Passman’s runtime configuration
#[derive(Serialize)]
pub struct ConfigDump {
    pub base_dir: String,
    pub prefix: String,
    pub secret: String,
    pub crypto_extension: String,
    pub public_key_filename: String,
    pub env: HashMap<String, String>,
    pub dependencies: HashMap<String, String>,
    /// If the vault is a git repo, the URL of the "origin" remote
    pub git_remote_origin: Option<String>,
}

pub fn extant_config() -> Result<ConfigDump> {
    // Load your existing Config struct
    let cfg: Config = load_config()?;

    // Capture selected environment variables
    let mut env_map = HashMap::new();
    for &key in &[
        "PASSMAN_DIR",
        "XDG_DATA_HOME",
        "PASSMAN_PUBLIC_KEY",
        "EDITOR",
    ] {
        if let Ok(val) = env::var(key) {
            env_map.insert(key.to_string(), val);
        }
    }

    // Probe for both encryption backends and git
    let mut deps = HashMap::new();
    for &tool in &["age", "rage", "git"] {
        let path = which(tool)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "<not found>".into());
        deps.insert(tool.to_string(), path);
    }

    // Detect a .git repo under the vault and clone its "origin" URL
    let git_remote_origin = if cfg.prefix.join(".git").is_dir() {
        match Repository::open(&cfg.prefix) {
            Ok(repo) => match repo.find_remote("origin") {
                Ok(remote) => remote.url().map(|s| s.to_string()),
                Err(_) => None,
            },
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(ConfigDump {
        base_dir: cfg.base_dir.display().to_string(),
        prefix: cfg.prefix.display().to_string(),
        secret: cfg.secret.display().to_string(),
        crypto_extension: cfg.crypto_extension,
        public_key_filename: cfg.public_key_filename,
        env: env_map,
        dependencies: deps,
        git_remote_origin,
    })
}
