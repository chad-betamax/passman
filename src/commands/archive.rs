use crate::config::Config;
use anyhow::{Context, Result};
use std::{fs, path::Path};

/// Hide a file by renaming it to a dot-prefixed hidden file in the same directory.
///
/// `path` is relative to the vault root (base_dir + prefix).
/// You can omit the crypto extension; it will be added automatically.
pub fn run(cfg: &Config, path: String) -> Result<()> {
    // Compute vault root
    let vault_root = cfg.base_dir.join(&cfg.prefix);

    // Resolve the user‐supplied path
    let mut full_path = vault_root.join(&path);

    // If no extension present, tack on the crypto_extension
    if full_path.extension().is_none() {
        let ext = cfg.crypto_extension.trim_start_matches('.');
        full_path.set_extension(ext);
    }

    // Existence & file‐type checks
    if !full_path.exists() {
        anyhow::bail!("File not found: {}", full_path.display());
    }
    if !full_path.is_file() {
        anyhow::bail!("Not a file: {}", full_path.display());
    }

    // Extract the basename (with extension)
    let file_name = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid UTF-8 in filename")?;

    // Prevent double‐archiving
    if file_name.starts_with('.') {
        anyhow::bail!("File already archived: {}", file_name);
    }

    // Build the new dot‐prefixed name
    let hidden_name = format!(".{}", file_name);
    let new_full = full_path.with_file_name(&hidden_name);

    // Perform the rename
    fs::rename(&full_path, &new_full).with_context(|| {
        format!(
            "Failed to archive {} → {}",
            full_path.display(),
            new_full.display()
        )
    })?;

    // Echo back only the relative path without extension
    let display_path = Path::new(&path).with_extension("");
    println!("Archived {}", display_path.display());

    Ok(())
}
