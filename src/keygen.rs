use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

pub fn generate_keypair(secret_path: &Path, public_path: &Path) -> Result<()> {
    // Ensure parent dirs exist
    if let Some(parent) = secret_path.parent() {
        fs::create_dir_all(parent).context("Creating parent directory for identities file")?;
    }

    // Generate secret key
    let status = Command::new("rage-keygen")
        .arg("-o")
        .arg(secret_path)
        .status()
        .context("Failed to run rage-keygen")?;

    if !status.success() {
        anyhow::bail!("rage-keygen failed");
    }

    // Extract public key
    let output = Command::new("rage-keygen")
        .arg("-y")
        .arg(secret_path)
        .output()
        .context("Failed to run rage-keygen -y")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to extract public key:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let public_key = String::from_utf8_lossy(&output.stdout);

    // Write to ~/.passman/recipient
    let mut file = File::create(public_path).context("Failed to create recipient file")?;
    file.write_all(public_key.as_bytes())
        .context("Failed to write public key to recipient file")?;

    println!("✅ Generated keypair:");
    println!("  🔐 Private:    {}", secret_path.display());
    println!("  🟢 Public: {}", public_path.display());
    println!(
        "\nCopy the public key to other systems to allow encryption to this identity:\n{}",
        public_key
    );

    Ok(())
}
