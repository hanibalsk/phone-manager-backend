//! Cryptographic utilities for API key generation and hashing.

use sha2::{Digest, Sha256};

/// Computes SHA-256 hash of the input and returns it as a hex string.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Extracts the prefix from an API key (first 8 characters after "pm_").
pub fn extract_key_prefix(key: &str) -> Option<&str> {
    if key.starts_with("pm_") && key.len() >= 11 {
        Some(&key[3..11])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex("test");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_extract_key_prefix() {
        assert_eq!(extract_key_prefix("pm_abcdefgh12345"), Some("abcdefgh"));
        assert_eq!(extract_key_prefix("pm_short"), None);
        assert_eq!(extract_key_prefix("invalid_key"), None);
    }
}
