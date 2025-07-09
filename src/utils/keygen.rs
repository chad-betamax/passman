use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use which::which;

pub fn generate_keypair(secret_path: &Path, public_path: &Path) -> Result<()> {
    let (bin, pub_flag, install_hint) = match secret_path.extension().and_then(OsStr::to_str) {
        Some("age") => (
            "age-keygen",
            "-y",
            "`age-keygen` not found. Please install age: https://github.com/FiloSottile/age",
        ),
        _ => (
            "rage-keygen",
            "-y",
            "`rage-keygen` not found. Please install rage: https://github.com/str4d/rage",
        ),
    };

    which(bin).context(install_hint)?;

    if let Some(parent) = secret_path.parent() {
        fs::create_dir_all(parent).context("Creating parent directory for identity file")?;
    }

    let status = Command::new(bin)
        .arg("-o")
        .arg(secret_path)
        .status()
        .with_context(|| format!("Failed to run `{}`", bin))?;

    if !status.success() {
        anyhow::bail!("{} failed to generate identity", bin);
    }

    let output = Command::new(bin)
        .arg(pub_flag)
        .arg(secret_path)
        .stdout(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to extract public key via `{}`", bin))?;

    if !output.status.success() {
        anyhow::bail!(
            "Public-key extraction failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let public_key = String::from_utf8_lossy(&output.stdout);

    let mut file = File::create(public_path).context("Failed to create recipient file")?;
    file.write_all(public_key.as_bytes())
        .context("Failed to write public key to recipient file")?;

    println!("✅ Generated {} keypair:", bin.trim_end_matches("-keygen"));
    println!("  🔐 Private:    {}", secret_path.display());
    println!("  🟢 Public:     {}", public_path.display());
    println!(
        "\nCopy the public key to other systems to allow encryption to this identity:\n{}",
        public_key
    );

    Ok(())
}
