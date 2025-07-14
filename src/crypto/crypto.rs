use anyhow::Result;
use std::path::Path;

use crate::crypto::age::Age; // ← now actually used below
use crate::crypto::backend::CryptoBackend;
use crate::crypto::rage::Rage; // ← now actually used below

/// Pick “Age” if the extension is `.age`, otherwise “Rage”
fn default_encrypt_backend(output: &Path) -> Box<dyn CryptoBackend> {
    if output.extension().and_then(|s| s.to_str()) == Some("age") {
        Box::new(Age)
    } else {
        Box::new(Rage)
    }
}

fn default_decrypt_backend(input: &Path) -> Box<dyn CryptoBackend> {
    if input.extension().and_then(|s| s.to_str()) == Some("age") {
        Box::new(Age)
    } else {
        Box::new(Rage)
    }
}

thread_local! {
    static ENCRYPT_FACTORY: std::cell::RefCell<fn(&Path) -> Box<dyn CryptoBackend>> =
        std::cell::RefCell::new(default_encrypt_backend);
    static DECRYPT_FACTORY: std::cell::RefCell<fn(&Path) -> Box<dyn CryptoBackend>> =
        std::cell::RefCell::new(default_decrypt_backend);
}

/// Exactly your old `encrypt`, but now driven by `ENCRYPT_FACTORY`
pub fn encrypt(recipient: &str, output_file: &Path, plaintext: &str) -> Result<()> {
    let backend = ENCRYPT_FACTORY.with(|f| (f.borrow())(output_file));
    backend.encrypt(recipient, output_file, plaintext)
}

/// Exactly your old `decrypt`, but now driven by `DECRYPT_FACTORY`
pub fn decrypt(identity_file: &Path, encrypted_file: &Path) -> Result<String> {
    let backend = DECRYPT_FACTORY.with(|f| (f.borrow())(encrypted_file));
    backend.decrypt(identity_file, encrypted_file)
}

/// Your existing detect‐backend facade
pub fn detect_backend() -> Result<String> {
    crate::crypto::detect::detect_crypto_backend()
}

/// Test hooks for injecting mocks
pub fn set_encrypt_factory(f: fn(&Path) -> Box<dyn CryptoBackend>) {
    ENCRYPT_FACTORY.with(|c| *c.borrow_mut() = f);
}

pub fn set_decrypt_factory(f: fn(&Path) -> Box<dyn CryptoBackend>) {
    DECRYPT_FACTORY.with(|c| *c.borrow_mut() = f);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::backend::CryptoBackend;
    use anyhow::Result;
    use std::path::Path;

    struct MockBackend;
    impl CryptoBackend for MockBackend {
        fn encrypt(&self, _r: &str, _o: &Path, plaintext: &str) -> Result<()> {
            assert_eq!(plaintext, "hello-test");
            Ok(())
        }
        fn decrypt(&self, _i: &Path, _e: &Path) -> Result<String> {
            Ok("mocked".into())
        }
    }

    fn mock_factory(_: &Path) -> Box<dyn CryptoBackend> {
        Box::new(MockBackend)
    }

    #[test]
    fn encrypt_is_mockable() -> Result<()> {
        set_encrypt_factory(mock_factory);
        encrypt("you", Path::new("foo.age"), "hello-test")?;
        Ok(())
    }

    #[test]
    fn decrypt_is_mockable() -> Result<()> {
        set_decrypt_factory(mock_factory);
        let out = decrypt(Path::new("id"), Path::new("foo.age"))?;
        assert_eq!(out, "mocked");
        Ok(())
    }
}
