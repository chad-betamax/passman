use crate::config::Config;
use crate::crypto;
use anyhow::Result;

pub fn run(config: &Config, path: String, clip: bool, qrcode: bool, line: usize) -> Result<()> {
    let file_path = config.prefix.join(format!("{}.rage", path));
    if !file_path.exists() {
        anyhow::bail!("No such password: {}", file_path.display());
    }

    let decrypted = crypto::decrypt(&config.secret, &file_path)?;

    if clip {
        println!("(Clipboard support not implemented yet)");
    } else if qrcode {
        println!("(QR code support not implemented yet)");
    } else if line > 1 {
        if let Some(line_str) = decrypted.lines().nth(line - 1) {
            println!("{}", line_str);
        } else {
            anyhow::bail!("File {} has fewer than {} lines", path, line);
        }
    } else {
        println!("{}", decrypted);
    }

    Ok(())
}
