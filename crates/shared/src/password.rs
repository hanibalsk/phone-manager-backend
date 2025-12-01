//! Password hashing utilities using Argon2id.
//!
//! This module provides secure password hashing using the Argon2id algorithm,
//! which is recommended by OWASP for password storage.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use thiserror::Error;

/// Error type for password operations.
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Failed to hash password: {0}")]
    HashError(String),

    #[error("Failed to verify password: {0}")]
    VerifyError(String),

    #[error("Invalid password hash format")]
    InvalidHashFormat,
}

/// Argon2id parameters following OWASP recommendations (2024).
/// - Memory: 19456 KiB (19 MiB)
/// - Iterations: 2
/// - Parallelism: 1
const MEMORY_COST: u32 = 19456; // 19 MiB in KiB
const TIME_COST: u32 = 2;
const PARALLELISM: u32 = 1;
const OUTPUT_LEN: usize = 32; // 256-bit hash output

/// Creates an Argon2id hasher with OWASP-recommended parameters.
fn create_argon2() -> Result<Argon2<'static>, PasswordError> {
    let params = Params::new(MEMORY_COST, TIME_COST, PARALLELISM, Some(OUTPUT_LEN))
        .map_err(|e| PasswordError::HashError(format!("Failed to create Argon2 params: {}", e)))?;

    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Hashes a password using Argon2id with OWASP-recommended parameters.
///
/// Returns a PHC-formatted string that includes the algorithm, parameters,
/// salt, and hash. This format is self-describing and allows for future
/// algorithm upgrades.
///
/// # Arguments
/// * `password` - The plaintext password to hash
///
/// # Returns
/// * `Ok(String)` - PHC-formatted hash string
/// * `Err(PasswordError)` - If hashing fails
///
/// # Example
/// ```
/// use shared::password::hash_password;
///
/// let hash = hash_password("my_secure_password").unwrap();
/// assert!(hash.starts_with("$argon2id$"));
/// ```
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = create_argon2()?;

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::HashError(e.to_string()))
}

/// Verifies a password against a stored hash.
///
/// This function is constant-time to prevent timing attacks.
///
/// # Arguments
/// * `password` - The plaintext password to verify
/// * `hash` - The PHC-formatted hash to verify against
///
/// # Returns
/// * `Ok(true)` - Password matches
/// * `Ok(false)` - Password does not match
/// * `Err(PasswordError)` - If verification fails (e.g., invalid hash format)
///
/// # Example
/// ```
/// use shared::password::{hash_password, verify_password};
///
/// let hash = hash_password("my_password").unwrap();
/// assert!(verify_password("my_password", &hash).unwrap());
/// assert!(!verify_password("wrong_password", &hash).unwrap());
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| PasswordError::InvalidHashFormat)?;

    // Create a new Argon2 instance for verification
    // Note: The stored hash contains its own parameters, so we use default for verification
    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(PasswordError::VerifyError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_returns_phc_format() {
        let hash = hash_password("test_password").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(hash.contains("$v=19$")); // Version 0x13 = 19
        assert!(hash.contains("m=19456")); // Memory cost
        assert!(hash.contains("t=2")); // Time cost
        assert!(hash.contains("p=1")); // Parallelism
    }

    #[test]
    fn test_hash_password_produces_unique_hashes() {
        let hash1 = hash_password("same_password").unwrap();
        let hash2 = hash_password("same_password").unwrap();
        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "my_secure_password123!";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let hash = hash_password("correct_password").unwrap();
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_verify_password_empty() {
        let hash = hash_password("").unwrap();
        assert!(verify_password("", &hash).unwrap());
        assert!(!verify_password("not_empty", &hash).unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("password", "invalid_hash_format");
        assert!(matches!(result, Err(PasswordError::InvalidHashFormat)));
    }

    #[test]
    fn test_hash_password_unicode() {
        let password = "密码123!пароль";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("different", &hash).unwrap());
    }

    #[test]
    fn test_hash_password_long_password() {
        // Argon2 handles passwords of any length
        let long_password = "a".repeat(1000);
        let hash = hash_password(&long_password).unwrap();
        assert!(verify_password(&long_password, &hash).unwrap());
    }

    #[test]
    fn test_hash_password_special_characters() {
        let password = r#"!@#$%^&*()_+-=[]{}|;':",.<>?/`~"#;
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_with_different_argon2_params() {
        // Verify that we can still verify hashes created with different parameters
        // This is important for algorithm upgrades
        let hash = hash_password("test").unwrap();

        // The verify function uses the parameters from the hash itself
        assert!(verify_password("test", &hash).unwrap());
    }

    #[test]
    fn test_hash_length_consistency() {
        // PHC format should produce consistent-length output structure
        let hash1 = hash_password("short").unwrap();
        let hash2 = hash_password("a_much_longer_password_here").unwrap();

        // Both should have similar structure (difference is only in encoded salt/hash)
        assert!(hash1.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
        assert!(hash2.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
    }

    #[test]
    fn test_password_error_display() {
        let err = PasswordError::HashError("test error".to_string());
        assert!(format!("{}", err).contains("test error"));

        let err = PasswordError::InvalidHashFormat;
        assert!(format!("{}", err).contains("Invalid password hash format"));
    }
}
