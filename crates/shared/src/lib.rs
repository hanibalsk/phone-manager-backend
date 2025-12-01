//! Shared utilities and common types for Phone Manager backend.
//!
//! This crate provides common functionality used across all other crates:
//! - Cryptographic utilities (hashing, key generation)
//! - Password hashing with Argon2id
//! - Common validation logic
//! - Shared error types

pub mod crypto;
pub mod pagination;
pub mod password;
pub mod validation;
