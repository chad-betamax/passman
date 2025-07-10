use crate::config::Config;
use crate::crypto;
use crate::utils::qr::print_qr;
use anyhow::Result;

/// Show a password entry, optionally only a single line.
pub fn run(
    config: &Config,
    path: String,
    clip: bool,
    qrcode: bool,
    line: Option<usize>,
) -> Result<()> {
    let file_path = config.prefix.join(format!("{}.rage", path));
    if !file_path.exists() {
        anyhow::bail!("No such password: {}", file_path.display());
    }

    let decrypted = crypto::decrypt(&config.secret, &file_path)?;

    // If `--line N` was provided (and N > 0), show that line; otherwise show all.
    let output = match line {
        None | Some(0) => decrypted.clone(),
        Some(n) => decrypted
            .lines()
            .nth(n - 1)
            .ok_or_else(|| anyhow::anyhow!("File {} has fewer than {} lines", path, n))?
            .to_string(),
    };

    if clip {
        println!("(Clipboard support not implemented yet)");
    } else if qrcode {
        print_qr(&output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}
