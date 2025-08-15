// FilePath: src/security/mod.rs

mod password;

pub use password::{EncryptedPassword, PasswordManager, PasswordSource};
