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
            qr,
            line,
        } => {
            // Show an entry
            commands::show::run(&cfg, path, qr, line)?;
        }

        Command::Show {
            config: false,
            path: None,
            ..
        } => {
            // This should never happen: clap enforces PATH unless --config
            unreachable!("`path` is required when `--config` is not set");
        }

        Command::New { path } => {
            commands::create::run(&cfg, path)?;
        }

        Command::Edit { path } => {
            commands::edit::run(&cfg, path)?;
        }

        Command::List { path, all } => {
            commands::list::run(&cfg, path, all)?;
        }

        Command::Archive { path, folder } => {
            commands::archive::run(&cfg, path, folder)?;
        }

        Command::Remove { path } => {
            commands::remove::run(&cfg, path)?;
        }
    }

    Ok(())
}
