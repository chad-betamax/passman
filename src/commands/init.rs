use crate::{completions, config::Config, utils::keygen};
use anyhow::Result;
use dialoguer::{Confirm, Input};
use std::{env, fs, process::Command};

pub fn run(config: &Config) -> Result<()> {
    let secret_path = &config.secret;
    let public_path = &config.base_dir.join("public.key");

    // 1) Keypair gen prompt
    let mut do_generate = true;
    if secret_path.exists() || public_path.exists() {
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
            fs::remove_file(secret_path).ok();
            fs::remove_file(public_path).ok();
        }
    }
    if do_generate {
        keygen::generate_keypair(secret_path, public_path)?;
    } else {
        println!("✅ Existing keypair remains intact.");
    }

    // 2) Git init + initial commit + remote + push (with pull-on-fail)
    let vault_dir = config.base_dir.join("vault");
    let git_dir = vault_dir.join(".git");

    if !git_dir.exists() {
        if Confirm::new()
            .with_prompt(format!(
                "No Git repo found at `{}`. Initialize one?",
                vault_dir.display()
            ))
            .default(true)
            .interact()?
        {
            fs::create_dir_all(&vault_dir)?;
            println!("🔧 git init in {}…", vault_dir.display());
            Command::new("git")
                .arg("init")
                .current_dir(&vault_dir)
                .status()?;

            // Rename default branch to 'main'
            let _ = Command::new("git")
                .arg("branch")
                .arg("-M")
                .arg("main")
                .current_dir(&vault_dir)
                .status();

            // Make initial commit
            let _ = Command::new("git")
                .arg("commit")
                .arg("--allow-empty")
                .arg("-m")
                .arg("Initial commit")
                .current_dir(&vault_dir)
                .status();

            // Prompt for remote
            let remote_url: String = Input::new()
                .with_prompt("Enter GitHub remote URL (SSH or HTTPS), or leave blank to skip")
                .allow_empty(true)
                .interact_text()?;
            if !remote_url.trim().is_empty() {
                // Add remote
                Command::new("git")
                    .arg("remote")
                    .arg("add")
                    .arg("origin")
                    .arg(remote_url.trim())
                    .current_dir(&vault_dir)
                    .status()?;
                println!("✅ remote 'origin' -> {}", remote_url.trim());

                // First push
                println!("🚀 Pushing 'main' and setting upstream…");
                let push = Command::new("git")
                    .arg("push")
                    .arg("--set-upstream")
                    .arg("origin")
                    .arg("main")
                    .current_dir(&vault_dir)
                    .status()?;

                if push.success() {
                    println!("✅ Pushed and set upstream to 'main'.");
                } else {
                    eprintln!("⚠️  Push failed (exit code {}).", push.code().unwrap_or(-1));
                    // Offer to pull & rebase
                    if Confirm::new()
                        .with_prompt("Remote contains commits you don’t have. Pull & rebase then retry push?")
                        .default(true)
                        .interact()?
                    {
                        println!("🔄 git pull --rebase origin main…");
                        let pull = Command::new("git")
                            .arg("pull")
                            .arg("--rebase")
                            .arg("origin")
                            .arg("main")
                            .current_dir(&vault_dir)
                            .status()?;
                        if pull.success() {
                            println!("✅ Pull/rebase succeeded. Retrying push…");
                            let retry = Command::new("git")
                                .arg("push")
                                .arg("--set-upstream")
                                .arg("origin")
                                .arg("main")
                                .current_dir(&vault_dir)
                                .status()?;
                            if retry.success() {
                                println!("✅ Successfully pushed after rebase.");
                            } else {
                                eprintln!("⚠️  Retry push still failed (code {}).", retry.code().unwrap_or(-1));
                            }
                        } else {
                            eprintln!("⚠️  Pull/rebase failed (code {}).", pull.code().unwrap_or(-1));
                        }
                    }
                }
            }
        } else {
            println!("ℹ️  Skipping Git initialization.");
        }
    } else {
        println!("✅ Git repository detected; continuing.");
    }

    // 3) Shell completions
    completions::install()?;
    let shell = env::var("SHELL").unwrap_or_default();
    let rc = if shell.ends_with("bash") {
        "~/.bashrc"
    } else if shell.ends_with("zsh") {
        "~/.zshrc"
    } else {
        "~/.bashrc (or ~/.zshrc)"
    };
    println!(
        "\n🔄 New completions installed!\n\
         source {}\n\
         (or add that to your {})",
        rc, rc
    );

    Ok(())
}
