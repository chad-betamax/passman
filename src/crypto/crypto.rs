use anyhow::Result;
use std::path::Path;

use crate::crypto::{CryptoBackend, age::Age, rage::Rage};

/// Pick backend by output filename extension
fn pick_encrypt_backend(output: &Path) -> Box<dyn CryptoBackend> {
    match output.extension().and_then(|s| s.to_str()) {
        Some("age") => Box::new(Age),
        _ => Box::new(Rage),
    }
}

/// Pick backend by input filename extension
fn pick_decrypt_backend(input: &Path) -> Box<dyn CryptoBackend> {
    match input.extension().and_then(|s| s.to_str()) {
        Some("age") => Box::new(Age),
        _ => Box::new(Rage),
    }
}

pub fn encrypt(recipient: &str, output_file: &Path, plaintext: &str) -> Result<()> {
    let backend = pick_encrypt_backend(output_file);
    backend.encrypt(recipient, output_file, plaintext)
}

pub fn decrypt(identity_file: &Path, encrypted_file: &Path) -> Result<String> {
    let backend = pick_decrypt_backend(encrypted_file);
    backend.decrypt(identity_file, encrypted_file)
}
