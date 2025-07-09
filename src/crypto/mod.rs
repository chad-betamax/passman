pub mod age;
pub mod backend;
pub mod rage;

use age::Age;
use anyhow::Result;
use backend::CryptoBackend;
use rage::Rage;
use std::path::Path;

fn pick_encrypt_backend(output: &Path) -> Box<dyn CryptoBackend> {
    match output.extension().and_then(|s| s.to_str()) {
        Some("age") => Box::new(Age),
        _ => Box::new(Rage),
    }
}

fn pick_decrypt_backend(input: &Path) -> Box<dyn CryptoBackend> {
    match input.extension().and_then(|s| s.to_str()) {
        Some("age") => Box::new(Age),
        _ => Box::new(Rage),
    }
}

/// Facade: caller just does `crypto::encrypt(...)`
pub fn encrypt(recipient: &str, output_file: &Path, plaintext: &str) -> Result<()> {
    pick_encrypt_backend(output_file).encrypt(recipient, output_file, plaintext)
}

/// Facade: caller just does `crypto::decrypt(...)`
pub fn decrypt(identity_file: &Path, encrypted_file: &Path) -> Result<String> {
    pick_decrypt_backend(encrypted_file).decrypt(identity_file, encrypted_file)
}
