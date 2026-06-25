// FilePath: src/security/mod.rs

#![forbid(unsafe_code)]

mod password;

pub use password::{EncryptedPassword, PasswordManager, PasswordSource};
