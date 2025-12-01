//! HTTP middleware components.

pub mod auth;
pub mod logging;
pub mod metrics;
pub mod rate_limit;
pub mod rbac;
pub mod security_headers;
pub mod trace_id;
pub mod user_auth;

#[allow(unused_imports)] // Re-exports for downstream use
pub use auth::{optional_auth, require_admin, require_auth};
#[allow(unused_imports)] // Re-exports for downstream use
pub use metrics::{init_metrics, metrics_handler, metrics_middleware};
#[allow(unused_imports)] // Re-exports for downstream use
pub use rate_limit::{rate_limit_middleware, RateLimiterState};
#[allow(unused_imports)] // Re-exports for downstream use
pub use rbac::{
    require_group_admin, require_group_member, require_group_owner, GroupMembership,
};
#[allow(unused_imports)] // Re-exports for downstream use
pub use security_headers::security_headers_middleware;
#[allow(unused_imports)] // Re-exports for downstream use
pub use trace_id::{trace_id, RequestId, REQUEST_ID_HEADER};
#[allow(unused_imports)] // Re-exports for downstream use
pub use user_auth::{optional_user_auth, require_user_auth, UserAuth};
