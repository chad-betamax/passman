use anyhow::Result;
use std::path::Path;

pub trait CryptoBackend {
    /// Decrypts the given file (using this backend) and returns its plaintext.
    fn decrypt(&self, identity_file: &Path, encrypted_file: &Path) -> Result<String>;

    /// Encrypts the given plaintext for the recipient and writes to `output_file`.
    fn encrypt(&self, recipient: &str, output_file: &Path, plaintext: &str) -> Result<()>;
}
