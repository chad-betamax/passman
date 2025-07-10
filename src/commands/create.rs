use crate::config::Config;
use crate::crypto;
use crate::utils::git_ops;
use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use tempfile::NamedTempFile;

pub fn run(config: &Config, path: String, prompt: bool, _echo: bool, force: bool) -> Result<()> {
    let output_path = config.entry_path(&path);

    if output_path.exists() && !force {
        anyhow::bail!(
            "Password already exists at {}. Use --force to overwrite.",
            output_path.display()
        );
    }

    // Ensure parent directories exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let password = if prompt {
        prompt_singleline(&path)?
    } else {
        edit_multiline()?
    };

    let public = config.read_public()?;
    crypto::encrypt(&public, &output_path, &password)?;
    println!("Password for '{}' stored successfully.", path);

    // Attempt to sync after successful write
    git_ops::sync_vault(&config.prefix)
}

fn prompt_singleline(path: &str) -> Result<String> {
    print!("Enter password for {}: ", path);
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    Ok(password.trim_end().to_string())
}

fn edit_multiline() -> Result<String> {
    let file = NamedTempFile::new().context("Failed to create temporary file")?;
    let path = file.path();

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor)
        .arg(path)
        .status()
        .context("Failed to launch $EDITOR")?;

    if !status.success() {
        anyhow::bail!("Editor exited with error");
    }

    let contents = fs::read_to_string(path)
        .context("Failed to read from temporary file")?
        .trim_end()
        .to_string();

    if contents.is_empty() {
        anyhow::bail!("Aborted: file was empty");
    }

    Ok(contents)
}
