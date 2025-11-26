//! HTTP middleware components.

pub mod auth;
pub mod logging;
pub mod trace_id;

#[allow(unused_imports)] // Re-exports for downstream use
pub use auth::{optional_auth, require_admin, require_auth};
#[allow(unused_imports)] // Re-exports for downstream use
pub use trace_id::{trace_id, RequestId, REQUEST_ID_HEADER};
