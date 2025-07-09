use crate::config::Config;
use crate::utils::git_sync;
use anyhow::{Context, Result};
use std::fs;

/// Delete a stored password entry
pub fn run(config: &Config, path: String) -> Result<()> {
    let file_path = config.prefix.join(format!("{}.rage", path));

    if !file_path.exists() {
        anyhow::bail!("No such entry: {}", file_path.display());
    }

    fs::remove_file(&file_path)
        .with_context(|| format!("Failed to delete file {:?}", file_path))?;

    // run from the vault dir ie the git root,
    // to pick up the deletion
    git_sync::sync_vault(&config.prefix)?;

    println!("✅ Removed entry `{}`", path);
    Ok(())
}
