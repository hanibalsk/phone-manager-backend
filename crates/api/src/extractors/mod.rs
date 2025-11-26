//! Custom Axum extractors.
//!
//! Extractors for parsing and validating request data.

pub mod api_key;

#[allow(unused_imports)] // Re-exports for downstream use
pub use api_key::{ApiKeyAuth, OptionalApiKeyAuth};
