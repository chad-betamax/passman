use crate::config::Config;
use crate::crypto;
use crate::utils::qr::print_qr;
use anyhow::Result;

pub fn run(config: &Config, path: String, clip: bool, qrcode: bool, line: usize) -> Result<()> {
    let file_path = config.prefix.join(format!("{}.rage", path));
    if !file_path.exists() {
        anyhow::bail!("No such password: {}", file_path.display());
    }

    let decrypted = crypto::decrypt(&config.secret, &file_path)?;

    let output = if line > 0 {
        decrypted
            .lines()
            .nth(line - 1)
            .ok_or_else(|| anyhow::anyhow!("File {} has fewer than {} lines", path, line))?
            .to_string()
    } else {
        decrypted.clone()
    };

    if clip {
        println!("(Clipboard support not implemented yet)");
    } else if qrcode {
        print_qr(&output)?;
    } else {
        println!("{output}");
    }

    Ok(())
}
