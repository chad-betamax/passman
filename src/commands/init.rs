// src/commands/init.rs

use crate::{completions, config::Config, utils::keygen};
use anyhow::Result;
use dialoguer::Confirm;
use std::fs;

pub fn run(config: &Config) -> Result<()> {
    let secret_path = &config.secret;
    let public_path = &config.base_dir.join("public.key");

    // Decide whether to generate new keys
    let mut do_generate = true;
    if secret_path.exists() || public_path.exists() {
        let prompt = format!(
            "Detected existing key files:\n  🔐 Private: {}\n  🟢 Public:  {}\nOverwrite?",
            secret_path.display(),
            public_path.display()
        );
        if !Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()?
        {
            println!("ℹ️  Keeping existing keys; skipping generation.");
            do_generate = false;
        } else {
            // remove old files so generate_keypair can recreate them cleanly
            fs::remove_file(secret_path).ok();
            fs::remove_file(public_path).ok();
        }
    }

    // Generate only if user agreed (or no keys existed)
    if do_generate {
        keygen::generate_keypair(secret_path, public_path)?;
    }

    // Always install completions
    completions::install()?;

    Ok(())
}
