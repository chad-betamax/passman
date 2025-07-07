mod cli;
mod commands;
mod completion;
mod config;
mod crypto;
mod keygen;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::load_config()?;

    match cli.command {
        Command::Show {
            path,
            clip,
            qrcode,
            line,
        } => {
            commands::show::run(&config, path, clip, qrcode, line)?;
        }
        Command::List { path } => {
            commands::list::run(&config, path)?;
        }
        Command::Insert {
            path,
            prompt,
            echo,
            force,
        } => {
            commands::insert::run(&config, path, prompt, echo, force)?;
        }
        Command::Init => {
            keygen::generate_keypair(&config.secret, &config.base_dir.join("public.key"))?;
            completion::install()?;
        }
    }

    Ok(())
}
