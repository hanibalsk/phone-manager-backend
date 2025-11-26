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
    fn test_sha256_hex_empty_string() {
        let hash = sha256_hex("");
        assert_eq!(hash.len(), 64);
        // SHA256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hex_deterministic() {
        let hash1 = sha256_hex("same_input");
        let hash2 = sha256_hex("same_input");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_sha256_hex_different_inputs() {
        let hash1 = sha256_hex("input1");
        let hash2 = sha256_hex("input2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_sha256_hex_long_input() {
        let long_input = "a".repeat(10000);
        let hash = sha256_hex(&long_input);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sha256_hex_unicode() {
        let hash = sha256_hex("你好世界");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_extract_key_prefix() {
        assert_eq!(extract_key_prefix("pm_abcdefgh12345"), Some("abcdefgh"));
        assert_eq!(extract_key_prefix("pm_short"), None);
        assert_eq!(extract_key_prefix("invalid_key"), None);
    }

    #[test]
    fn test_extract_key_prefix_exact_length() {
        // pm_ (3) + 8 characters = 11 minimum
        assert_eq!(extract_key_prefix("pm_12345678"), Some("12345678"));
    }

    #[test]
    fn test_extract_key_prefix_too_short() {
        assert_eq!(extract_key_prefix("pm_1234567"), None); // Only 7 chars after pm_
        assert_eq!(extract_key_prefix("pm_"), None);
        assert_eq!(extract_key_prefix("pm"), None);
    }

    #[test]
    fn test_extract_key_prefix_wrong_prefix() {
        assert_eq!(extract_key_prefix("sk_abcdefgh12345"), None);
        assert_eq!(extract_key_prefix("PM_abcdefgh12345"), None); // Case sensitive
        assert_eq!(extract_key_prefix("pM_abcdefgh12345"), None);
    }

    #[test]
    fn test_extract_key_prefix_empty() {
        assert_eq!(extract_key_prefix(""), None);
    }

    #[test]
    fn test_extract_key_prefix_long_key() {
        let long_key = format!("pm_{}", "x".repeat(100));
        assert_eq!(extract_key_prefix(&long_key), Some("xxxxxxxx"));
    }
}
