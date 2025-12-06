//! Client version compatibility middleware.
//!
//! Extracts X-Client-Version header and logs warnings for clients
//! with versions below the minimum compatible version.
//! This is a soft enforcement - requests are never rejected.

use axum::{body::Body, http::Request, middleware::Next, response::Response};
use tracing::warn;

/// Header name for client version.
pub const CLIENT_VERSION_HEADER: &str = "X-Client-Version";

/// Minimum compatible client version.
/// Clients below this version will trigger warnings but NOT be rejected.
pub const MIN_COMPATIBLE_VERSION: &str = "0.8.0";

/// Current server version from Cargo.toml.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Middleware that checks client version compatibility.
///
/// Logs warnings for clients with incompatible versions but allows
/// requests to proceed (soft enforcement).
pub async fn version_check(req: Request<Body>, next: Next) -> Response {
    // Extract client version from header
    if let Some(client_version) = req.headers().get(CLIENT_VERSION_HEADER) {
        if let Ok(version_str) = client_version.to_str() {
            // Simple semver comparison using string comparison
            // This works for versions like "0.8.0", "0.9.0", "1.0.0"
            if is_version_below_minimum(version_str, MIN_COMPATIBLE_VERSION) {
                warn!(
                    client_version = %version_str,
                    server_version = SERVER_VERSION,
                    min_required = MIN_COMPATIBLE_VERSION,
                    path = %req.uri().path(),
                    "Client version below minimum compatible version"
                );
            }
        }
    }
    // Always proceed - this is soft enforcement only
    next.run(req).await
}

/// Compare two semantic versions.
/// Returns true if version is below minimum.
fn is_version_below_minimum(version: &str, minimum: &str) -> bool {
    let parse_version = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() >= 3 {
            Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            ))
        } else {
            None
        }
    };

    match (parse_version(version), parse_version(minimum)) {
        (Some((v_major, v_minor, v_patch)), Some((m_major, m_minor, m_patch))) => {
            (v_major, v_minor, v_patch) < (m_major, m_minor, m_patch)
        }
        _ => false, // If parsing fails, don't warn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        // Below minimum
        assert!(is_version_below_minimum("0.7.0", "0.8.0"));
        assert!(is_version_below_minimum("0.7.9", "0.8.0"));
        assert!(is_version_below_minimum("0.0.1", "0.8.0"));

        // At or above minimum
        assert!(!is_version_below_minimum("0.8.0", "0.8.0"));
        assert!(!is_version_below_minimum("0.8.1", "0.8.0"));
        assert!(!is_version_below_minimum("0.9.0", "0.8.0"));
        assert!(!is_version_below_minimum("1.0.0", "0.8.0"));
        assert!(!is_version_below_minimum("1.2.3", "0.8.0"));
    }

    #[test]
    fn test_invalid_versions() {
        // Invalid formats should not cause warnings
        assert!(!is_version_below_minimum("invalid", "0.8.0"));
        assert!(!is_version_below_minimum("1.0", "0.8.0"));
        assert!(!is_version_below_minimum("", "0.8.0"));
    }
}
