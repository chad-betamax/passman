use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

/// Decrypts a `.rage` file using the provided identity file.
pub fn decrypt(identity_file: &Path, encrypted_file: &Path) -> Result<String> {
    let output = Command::new("rage")
        .arg("-d")
        .arg("-i")
        .arg(identity_file)
        .arg(encrypted_file)
        .stdout(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to run `rage -d` on {}", encrypted_file.display()))?;

    if !output.status.success() {
        anyhow::bail!(
            "rage decryption failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Encrypts given plaintext using a recipient and writes to output file.
pub fn encrypt(recipient: &str, output_file: &Path, plaintext: &str) -> Result<()> {
    let mut child = Command::new("rage")
        .arg("-r")
        .arg(recipient)
        .arg("-o")
        .arg(output_file)
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to start rage encryption")?;

    use std::io::Write;
    if let Some(stdin) = &mut child.stdin {
        stdin
            .write_all(plaintext.as_bytes())
            .context("Failed to write plaintext to rage stdin")?;
    }

    let status = child.wait().context("Failed to wait on rage process")?;
    if !status.success() {
        anyhow::bail!("rage encryption failed");
    }

    Ok(())
}
