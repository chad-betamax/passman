use crate::config::Config;
use crate::crypto;
use crate::utils::git_ops;
use anyhow::{Context, Result};
use std::{env, fs, process::Command};
use tempfile::NamedTempFile;

/// Edit an existing password entry in your $EDITOR and re‐encrypt it.
pub fn run(config: &Config, path: String) -> Result<()> {
    // Build the path to the encrypted file
    let file_path = config.prefix.join(format!("{}.rage", path));
    if !file_path.exists() {
        anyhow::bail!("No such password: {}", file_path.display());
    }

    // Decrypt using the private key
    let plaintext =
        crypto::decrypt(&config.secret, &file_path).context("Failed to decrypt existing entry")?;

    // Create a temp file with the decrypted contents
    let tmp = NamedTempFile::new().context("Failed to create temporary file")?;
    fs::write(tmp.path(), &plaintext)
        .with_context(|| format!("Failed to write to temporary file {:?}", tmp.path()))?;

    // Launch the editor
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor)
        .arg(tmp.path())
        .status()
        .context("Failed to launch $EDITOR")?;
    if !status.success() {
        anyhow::bail!("Editor exited with an error");
    }

    // Read back the edited contents
    let updated = fs::read_to_string(tmp.path())
        .context("Failed to read from temporary file")?
        .trim_end()
        .to_string();
    if updated.is_empty() {
        anyhow::bail!("Aborted: no content (file was empty)");
    }

    // Load the public key
    let public_path = config.base_dir.join("public.key");
    let public = fs::read_to_string(&public_path)
        .with_context(|| format!("Failed to read public key: {}", public_path.display()))?
        .trim()
        .to_string();

    // Re‐encrypt back into the same file
    crypto::encrypt(&public, &file_path, &updated).context("Failed to re-encrypt updated entry")?;
    println!("Password for '{}' updated successfully.", path);

    // Sync the vault via Git
    git_ops::sync_vault(&config.prefix)?;

    Ok(())
}
