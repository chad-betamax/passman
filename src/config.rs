use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

use crate::crypto::detect_backend;

pub struct Config {
    /// Where we keep our vault, key, etc.
    pub base_dir: PathBuf,

    /// `<base_dir>/vault`
    pub prefix: PathBuf,

    /// `<base_dir>/private.rage` or `.age`
    pub secret: PathBuf,

    /// File extension for encrypted entries, e.g. "age" or "rage"
    pub crypto_extension: String,

    /// Name of the public key file, e.g. "public.key"
    pub public_key_filename: String,
}

impl Config {
    /// Return the path to the encrypted entry for a given name
    pub fn entry_path(&self, name: &str) -> PathBuf {
        self.prefix
            .join(format!("{}.{}", name, self.crypto_extension))
    }

    /// Read and trim the public key file from disk
    pub fn read_public(&self) -> Result<String> {
        let path = self.base_dir.join(&self.public_key_filename);
        fs::read_to_string(&path)
            .with_context(|| format!("Failed to read public key: {}", path.display()))
            .map(|s| s.trim().to_string())
    }
}

pub fn load_config() -> Result<Config> {
    // Determine base dir
    let base_dir = if let Ok(dir) = env::var("PASSMAN_DIR") {
        PathBuf::from(dir)
    } else {
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

    // Determine crypto backend: rage or age
    let crypto_extension = detect_backend()?;

    // Construct paths
    let prefix = base_dir.join("vault");
    let secret = base_dir.join(format!("private.{}", crypto_extension));

    // Ensure vault dir exists
    fs::create_dir_all(&prefix).context("Failed to create passman vault directory")?;

    // Public key filename override
    let public_key_filename =
        env::var("PASSMAN_PUBLIC_KEY").unwrap_or_else(|_| "public.key".to_string());

    Ok(Config {
        base_dir,
        prefix,
        secret,
        crypto_extension,
        public_key_filename,
    })
}
