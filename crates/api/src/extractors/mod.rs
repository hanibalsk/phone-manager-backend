//! Custom Axum extractors.
//!
//! Extractors for parsing and validating request data.

pub mod api_key;
pub mod idempotency_key;
pub mod user_auth;

#[allow(unused_imports)] // Re-exports for downstream use
pub use api_key::{ApiKeyAuth, OptionalApiKeyAuth};
#[allow(unused_imports)] // Re-exports for downstream use
pub use idempotency_key::{IdempotencyKey, OptionalIdempotencyKey, IDEMPOTENCY_KEY_HEADER};
#[allow(unused_imports)] // Re-exports for downstream use
pub use user_auth::{OptionalUserAuth, UserAuth};
