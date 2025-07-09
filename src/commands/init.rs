use crate::{completions, config::Config, utils::keygen};
use anyhow::Result;
use dialoguer::Confirm;
use std::{env, fs};

pub fn run(config: &Config) -> Result<()> {
    let secret_path = &config.secret;
    let public_path = &config.base_dir.join("public.key");

    // Should we generate a fresh keypair?
    let mut do_generate = true;
    if secret_path.exists() || public_path.exists() {
        // ⚠️ Stronger warning!
        let prompt = format!(
            "⚠️  DANGER: Overwriting your secret key is irreversible!\n\
    🔐 Private key: {}\n\
    🟢 Public key:  {}\n\n\
Any files encrypted with your *current* private key will become PERMANENTLY inaccessible!\n\
Do you REALLY want to overwrite these files?",
            secret_path.display(),
            public_path.display()
        );

        if !Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()?
        {
            println!("ℹ️  Keeping existing keys; skipping key generation.");
            do_generate = false;
        } else {
            // remove old files so we start clean
            fs::remove_file(secret_path).ok();
            fs::remove_file(public_path).ok();
        }
    }

    if do_generate {
        keygen::generate_keypair(secret_path, public_path)?;
    } else {
        println!("✅ Existing keypair remains intact.");
    }

    // shell completions always go ahead
    completions::install()?;

    // Remind user to reload their shell to pick up new completions
    //    Detect whether they're using bash or zsh
    let shell = env::var("SHELL").unwrap_or_default();
    let rc = if shell.ends_with("bash") {
        "~/.bashrc"
    } else if shell.ends_with("zsh") {
        "~/.zshrc"
    } else {
        "~/.bashrc (or ~/.zshrc)"
    };

    println!(
        "\n🔄 New shell completions installed!\n\
         To enable them right now, run:\n\
           source {}\n\n\
         You can also add that line into your {},\n\
         so completions load automatically in every new shell session.",
        rc, rc
    );

    Ok(())
}
