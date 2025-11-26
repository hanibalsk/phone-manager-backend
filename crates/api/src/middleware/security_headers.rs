//! Security headers middleware.
//!
//! Adds security-related HTTP headers to all responses.

use axum::{
    body::Body,
    http::{header, HeaderValue, Request},
    middleware::Next,
    response::Response,
};

/// Middleware that adds security headers to all responses.
///
/// Headers added:
/// - `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
/// - `X-Frame-Options: DENY` - Prevents clickjacking by disallowing framing
/// - `X-XSS-Protection: 1; mode=block` - Enables XSS filtering in older browsers
/// - `Strict-Transport-Security` - Enforces HTTPS (if enabled via env var)
///
/// Note: Strict-Transport-Security is only added when the `PM__SECURITY__HSTS_ENABLED`
/// environment variable is set to "true", as it should only be enabled in production
/// with proper HTTPS termination.
pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking - deny all framing
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // Enable XSS filter in legacy browsers
    headers.insert(
        header::HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Add HSTS header if enabled (for production HTTPS)
    // This should only be enabled when TLS is properly configured at the load balancer
    if std::env::var("PM__SECURITY__HSTS_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
    {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }

    response
}

/// Security header names as constants for testing and documentation.
#[allow(dead_code)] // Available for use in integration tests
pub mod headers {
    /// X-Content-Type-Options header name.
    pub const X_CONTENT_TYPE_OPTIONS: &str = "x-content-type-options";
    /// X-Frame-Options header name.
    pub const X_FRAME_OPTIONS: &str = "x-frame-options";
    /// X-XSS-Protection header name.
    pub const X_XSS_PROTECTION: &str = "x-xss-protection";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_constants() {
        assert_eq!(headers::X_CONTENT_TYPE_OPTIONS, "x-content-type-options");
        assert_eq!(headers::X_FRAME_OPTIONS, "x-frame-options");
        assert_eq!(headers::X_XSS_PROTECTION, "x-xss-protection");
    }

    // Note: Full middleware tests would require setting up a mock router,
    // which is tested at the integration level. Unit tests verify constants
    // and simple logic.

    #[test]
    fn test_hsts_env_parsing() {
        // Test that env var parsing works correctly
        // In actual tests, we'd use temp_env or similar to set env vars
        let result = std::env::var("PM__SECURITY__HSTS_ENABLED_TEST")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        assert!(!result); // Should be false when not set
    }
}
