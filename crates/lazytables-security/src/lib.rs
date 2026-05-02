// FilePath: crates/lazytables-security/src/lib.rs

// Password encryption and decryption for LazyTables using AES-256-GCM
// with Argon2 key derivation for secure at-rest credential storage.

pub use lazytables_core::security::{EncryptedPassword, PasswordSource};

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{password_hash::rand_core::RngCore, Argon2};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::env;

/// Password manager for secure password handling
pub struct PasswordManager;

impl PasswordManager {
    /// Derive encryption key from user password using Argon2
    fn derive_key(password: &str, salt: &[u8]) -> Result<Key<Aes256Gcm>, String> {
        let argon2 = Argon2::default();
        let mut key_bytes = [0u8; 32];

        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key_bytes)
            .map_err(|e| format!("Failed to derive key: {e}"))?;

        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }

    /// Encrypt a password with a user-provided encryption key
    pub fn encrypt_password(
        plaintext: &str,
        encryption_key: &str,
        hint: Option<String>,
    ) -> Result<EncryptedPassword, String> {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        let key = Self::derive_key(encryption_key, &salt)?;
        let cipher = Aes256Gcm::new(&key);

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {e}"))?;

        Ok(EncryptedPassword {
            ciphertext: BASE64.encode(&ciphertext),
            nonce: BASE64.encode(nonce_bytes),
            salt: BASE64.encode(salt),
            hint,
        })
    }

    /// Decrypt a password with a user-provided encryption key
    pub fn decrypt_password(
        encrypted: &EncryptedPassword,
        encryption_key: &str,
    ) -> Result<String, String> {
        let ciphertext = BASE64
            .decode(&encrypted.ciphertext)
            .map_err(|e| format!("Invalid ciphertext: {e}"))?;
        let nonce_bytes = BASE64
            .decode(&encrypted.nonce)
            .map_err(|e| format!("Invalid nonce: {e}"))?;
        let salt = BASE64
            .decode(&encrypted.salt)
            .map_err(|e| format!("Invalid salt: {e}"))?;

        let key = Self::derive_key(encryption_key, &salt)?;
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "Decryption failed - incorrect encryption key")?;

        String::from_utf8(plaintext)
            .map_err(|e| format!("Invalid UTF-8 in decrypted password: {e}"))
    }

    /// Resolve a password from its source
    pub fn resolve_password(
        source: &PasswordSource,
        encryption_key: Option<&str>,
    ) -> Result<String, String> {
        match source {
            PasswordSource::Environment { var_name } => env::var(var_name)
                .map_err(|_| format!("Environment variable '{var_name}' not found")),
            PasswordSource::Encrypted(encrypted) => {
                let key = encryption_key.ok_or("Encryption key required for encrypted password")?;
                Self::decrypt_password(encrypted, key)
            }
            PasswordSource::PlainText(password) => Ok(password.clone()),
        }
    }

    /// Create a password source from environment variable
    pub fn from_environment(var_name: String) -> PasswordSource {
        PasswordSource::Environment { var_name }
    }

    /// Create an encrypted password source
    pub fn create_encrypted(
        password: &str,
        encryption_key: &str,
        hint: Option<String>,
    ) -> Result<PasswordSource, String> {
        let encrypted = Self::encrypt_password(password, encryption_key, hint)?;
        Ok(PasswordSource::Encrypted(encrypted))
    }

    /// Check if a password source requires an encryption key
    pub fn requires_encryption_key(source: &PasswordSource) -> bool {
        matches!(source, PasswordSource::Encrypted(_))
    }

    /// Get hint for encrypted password
    pub fn get_hint(source: &PasswordSource) -> Option<String> {
        match source {
            PasswordSource::Encrypted(encrypted) => encrypted.hint.clone(),
            _ => None,
        }
    }

    /// Migrate plain text password to encrypted
    pub fn migrate_to_encrypted(
        plaintext: &str,
        encryption_key: &str,
        hint: Option<String>,
    ) -> Result<PasswordSource, String> {
        Self::create_encrypted(plaintext, encryption_key, hint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "my_database_password";
        let encryption_key = "my_secret_key_123";
        let hint = Some("Test hint".to_string());

        let encrypted = PasswordManager::encrypt_password(password, encryption_key, hint.clone())
            .expect("Encryption should succeed");

        assert_eq!(encrypted.hint, hint);

        let decrypted = PasswordManager::decrypt_password(&encrypted, encryption_key)
            .expect("Decryption should succeed");

        assert_eq!(decrypted, password);
    }

    #[test]
    fn test_wrong_key_fails() {
        let password = "my_database_password";
        let encryption_key = "my_secret_key_123";
        let wrong_key = "wrong_key";

        let encrypted = PasswordManager::encrypt_password(password, encryption_key, None)
            .expect("Encryption should succeed");

        let result = PasswordManager::decrypt_password(&encrypted, wrong_key);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Decryption failed"));
    }

    #[test]
    fn test_environment_variable() {
        let var_name = "TEST_DB_PASSWORD_SECURITY_CRATE";
        let password = "test_password";

        unsafe { std::env::set_var(var_name, password) };

        let source = PasswordManager::from_environment(var_name.to_string());

        let resolved = PasswordManager::resolve_password(&source, None)
            .expect("Should resolve from environment");

        assert_eq!(resolved, password);

        unsafe { std::env::remove_var(var_name) };
    }

    #[test]
    fn test_password_source_serialization() {
        let password = "test";
        let key = "key";

        let source = PasswordManager::create_encrypted(password, key, Some("hint".to_string()))
            .expect("Should create encrypted source");

        let json = serde_json::to_string(&source).expect("Should serialize");

        let deserialized: PasswordSource = serde_json::from_str(&json).expect("Should deserialize");

        let resolved = PasswordManager::resolve_password(&deserialized, Some(key))
            .expect("Should resolve password");

        assert_eq!(resolved, password);
    }
}
