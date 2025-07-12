use crate::config::Config;
use anyhow::{Context, Result};
use std::{fs, path::Path};

/// Hide a file or directory by renaming it to a dot‐prefixed hidden entry in the same directory.
///
/// Usage:
///     passman archive [--folder] <path>
///
/// `path` is relative to the vault root (base_dir + prefix).
/// When archiving a file, you can omit the crypto extension; it will be added automatically.
/// When `--folder` is set, `path` must refer to a directory (no extension logic applied).
pub fn run(cfg: &Config, path: String, folder: bool) -> Result<()> {
    // Compute vault root
    let vault_root = cfg.base_dir.join(&cfg.prefix);
    let mut full_path = vault_root.join(&path);

    // If we're archiving a file and no extension present, tack on the crypto_extension
    if !folder && full_path.extension().is_none() {
        let ext = cfg.crypto_extension.trim_start_matches('.');
        full_path.set_extension(ext);
    }

    // Existence check
    if !full_path.exists() {
        anyhow::bail!("Not found: {}", full_path.display());
    }
    // Type check
    if folder {
        if !full_path.is_dir() {
            anyhow::bail!("Not a directory: {}", full_path.display());
        }
    } else {
        if !full_path.is_file() {
            anyhow::bail!("Not a file: {}", full_path.display());
        }
    }

    // Extract the basename (with extension, if any)
    let name = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid UTF-8 in name")?;

    // Prevent double‐archiving
    if name.starts_with('.') {
        anyhow::bail!(
            "{} already archived",
            if folder { "Directory" } else { "File" }
        );
    }

    // Build the new dot‐prefixed name
    let hidden_name = format!(".{}", name);
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
    if folder {
        println!("Archived folder {}", display_path.display());
    } else {
        println!("Archived {}", display_path.display());
    }

    Ok(())
}
