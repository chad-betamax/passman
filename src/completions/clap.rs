use crate::cli::Cli;
use clap::CommandFactory;
use clap_complete::{generate_to, shells::Bash};
use std::fs;
use std::path::PathBuf;

pub fn generate_completion_script() -> anyhow::Result<()> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bash/completions");

    fs::create_dir_all(&dir)?;

    let mut cmd = Cli::command(); // <- This gives a Command
    let path = generate_to(Bash, &mut cmd, "passman", &dir)?;
    rename_completion_function(&path)?;

    Ok(())
}

// Post-process the completion file to rename `_passman` -> `_passman_clap`
fn rename_completion_function(path: &PathBuf) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(&path)?;
    let replaced = contents.replace("_passman", "_passman_clap");
    std::fs::write(&path, replaced)?;
    Ok(())
}
