// FilePath: src/security/mod.rs

// Security module for credential management. Provides AES-GCM encryption with
// Argon2 key derivation for secure database password storage.

mod password;

pub use password::{EncryptedPassword, PasswordManager, PasswordSource};
