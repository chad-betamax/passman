mod cli;
mod commands;
mod completions;
mod config;
mod crypto;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::load_config()?;

    match cli.command {
        Command::Init => {
            commands::init::run(&config)?;
        }
        Command::Show {
            path,
            clip,
            qr,
            line,
        } => {
            commands::show::run(&config, path, clip, qr, line)?;
        }
        Command::List { path } => {
            commands::list::run(&config, path)?;
        }
        Command::New {
            path,
            prompt,
            echo,
            force,
        } => {
            commands::create::run(&config, path, prompt, echo, force)?;
        }
        Command::Remove { path } => {
            commands::remove::run(&config, path)?;
        }
    }

    Ok(())
}
