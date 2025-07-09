use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};

/// Runs a silent Git command. On real execution errors, prints warning but returns Ok(()).
fn git_silent(repo_path: &Path, label: &str, args: &[&str]) -> Result<()> {
    let result = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .envs(std::env::vars())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Ok(status) if !status.success() => {
            println!(
                "⚠️  git {label} failed (exit code {})",
                status.code().unwrap_or(-1)
            );
            Ok(())
        }
        Ok(_) => Ok(()),
        Err(e) => {
            println!("⚠️  git {label} failed to run: {e}");
            Ok(())
        }
    }
}

/// Attempts to sync the vault repo. All errors are printed but never cause failure.
pub fn sync_vault(repo_path: &Path) -> Result<()> {
    println!("🔄 syncing with GitHub...");

    // stage all changes **including** deletions
    git_silent(repo_path, "add", &["add", "."])?;

    git_silent(repo_path, "commit", &["commit", "-m", "Sync vault"])?;
    git_silent(repo_path, "pull", &["pull", "--rebase"])?;
    git_silent(repo_path, "push", &["push"])?;

    Ok(())
}
