pub mod age;
pub mod backend;
pub mod crypto;
pub mod detect;
pub mod rage;

// Public façade for normal code:
pub use crypto::{decrypt, detect_backend, encrypt};

/// Test-only exports:
#[cfg(test)]
pub use crypto::{set_decrypt_factory, set_encrypt_factory};

#[cfg(test)]
pub use backend::CryptoBackend;
