// FilePath: crates/lazytables-core/src/security.rs

// Type definitions for password sources and encrypted password storage.
// The actual encryption/decryption logic lives in lazytables-security.

use serde::{Deserialize, Serialize};

/// Password source - environment variable or encrypted storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasswordSource {
    /// Password comes from environment variable
    Environment {
        /// Name of the environment variable
        var_name: String,
    },
    /// Password is stored encrypted
    Encrypted(EncryptedPassword),
    /// Password is stored in plain text (deprecated, for migration only)
    PlainText(String),
}

impl PasswordSource {
    /// Check if this password source requires an encryption key
    pub fn requires_encryption_key(&self) -> bool {
        matches!(self, Self::Encrypted(_))
    }

    /// Get hint for encrypted password
    pub fn get_hint(&self) -> Option<String> {
        match self {
            Self::Encrypted(encrypted) => encrypted.hint.clone(),
            _ => None,
        }
    }
}

/// Encrypted password storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPassword {
    /// Encrypted password data (base64 encoded)
    pub ciphertext: String,
    /// Nonce used for encryption (base64 encoded)
    pub nonce: String,
    /// Salt used for key derivation (base64 encoded)
    pub salt: String,
    /// Hint for the encryption key (optional)
    pub hint: Option<String>,
}
