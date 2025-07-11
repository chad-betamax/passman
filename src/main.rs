mod cli;
mod commands;
mod completions;
mod config;
mod crypto;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use serde_json::to_string_pretty;
use utils::gather_config::extant_config;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load_config()?;

    match cli.command {
        Command::Init => {
            commands::init::run(&cfg)?;
        }

        Command::Show { config: true, .. } => {
            // Dump runtime config
            let dump = extant_config()?;
            println!("{}", to_string_pretty(&dump)?);
        }

        Command::Show {
            config: false,
            path: Some(path),
            clip,
            qr,
            line,
        } => {
            // Show an entry
            commands::show::run(&cfg, path, clip, qr, line)?;
        }

        Command::Show {
            config: false,
            path: None,
            ..
        } => {
            // This should never happen: clap enforces PATH unless --config
            unreachable!("`path` is required when `--config` is not set");
        }

        Command::New {
            path,
            prompt,
            echo,
            force,
        } => {
            commands::create::run(&cfg, path, prompt, echo, force)?;
        }

        Command::Edit { path } => {
            commands::edit::run(&cfg, path)?;
        }
        Command::List { path, all } => {
            commands::list::run(&cfg, path, all)?;
        }

        Command::Archive { path } => {
            // Archive an entry by renaming it to a dot-prefixed hidden file
            commands::archive::run(&cfg, path)?;
        }

        Command::Remove { path } => {
            commands::remove::run(&cfg, path)?;
        }
    }

    Ok(())
}
