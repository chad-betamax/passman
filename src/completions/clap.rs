use crate::cli::Cli;
use clap::CommandFactory;
use clap_complete::{generate_to, shells::Bash};
use std::fs;
use std::path::PathBuf;

pub fn generate_completion_script(bin_name: &str) -> anyhow::Result<()> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bash/completions");

    fs::create_dir_all(&dir)?;

    let mut cmd = Cli::command();
    // feed our dynamic bin_name into the generator
    let path = generate_to(Bash, &mut cmd, bin_name, &dir)?;
    rename_completion_function(&path, bin_name)?;
    Ok(())
}

// Rename all instances of `_bin_name` → `_{bin_name}_clap`
fn rename_completion_function(path: &PathBuf, bin_name: &str) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(&path)?;
    let old_fn = format!("_{}", bin_name);
    let new_fn = format!("_{}_clap", bin_name);
    let replaced = contents.replace(&old_fn, &new_fn);
    std::fs::write(path, replaced)?;
    Ok(())
}
