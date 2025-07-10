use anyhow::{Result, bail};
use std::io::{self, Write};
use which::which;

/// Detects which crypto backend to use: "rage" or "age"
pub fn detect_crypto_backend() -> Result<String> {
    let has_rage = which("rage").is_ok();
    let has_age = which("age").is_ok();

    match (has_rage, has_age) {
        (false, false) => {
            bail!(
                "❌ Neither `rage` nor `age` is installed. Please install one:\n\
                 - rage: https://github.com/str4d/rage\n\
                 - age:  https://github.com/FiloSottile/age"
            );
        }
        (true, false) => Ok("rage".to_string()),
        (false, true) => Ok("age".to_string()),
        (true, true) => {
            eprintln!("🛠 Both `rage` and `age` are installed.");
            loop {
                eprint!("👉 Which do you want to use? [rage/age]: ");
                io::stdout().flush().ok();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    match input.trim().to_lowercase().as_str() {
                        "rage" => return Ok("rage".to_string()),
                        "age" => return Ok("age".to_string()),
                        _ => eprintln!("❓ Please type either 'rage' or 'age'."),
                    }
                }
            }
        }
    }
}
