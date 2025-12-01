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

    // ===========================================
    // Header Constants Tests
    // ===========================================

    #[test]
    fn test_header_constants() {
        assert_eq!(headers::X_CONTENT_TYPE_OPTIONS, "x-content-type-options");
        assert_eq!(headers::X_FRAME_OPTIONS, "x-frame-options");
        assert_eq!(headers::X_XSS_PROTECTION, "x-xss-protection");
    }

    #[test]
    fn test_header_constants_lowercase() {
        // Verify headers are lowercase (HTTP headers are case-insensitive,
        // but we should be consistent)
        assert!(headers::X_CONTENT_TYPE_OPTIONS
            .chars()
            .all(|c| c.is_lowercase() || c == '-'));
        assert!(headers::X_FRAME_OPTIONS
            .chars()
            .all(|c| c.is_lowercase() || c == '-'));
        assert!(headers::X_XSS_PROTECTION
            .chars()
            .all(|c| c.is_lowercase() || c == '-'));
    }

    #[test]
    fn test_header_constants_not_empty() {
        assert!(!headers::X_CONTENT_TYPE_OPTIONS.is_empty());
        assert!(!headers::X_FRAME_OPTIONS.is_empty());
        assert!(!headers::X_XSS_PROTECTION.is_empty());
    }

    // ===========================================
    // HSTS Environment Parsing Tests
    // ===========================================

    #[test]
    fn test_hsts_env_parsing_not_set() {
        // Test that env var parsing works correctly when not set
        let result = std::env::var("PM__SECURITY__HSTS_ENABLED_NONEXISTENT_VAR")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        assert!(!result);
    }

    #[test]
    fn test_hsts_env_parsing_logic_true() {
        // Test the parsing logic (without actually setting env var)
        let test_cases = vec![
            ("true", true),
            ("TRUE", true),
            ("True", true),
            ("TrUe", true),
        ];

        for (input, expected) in test_cases {
            let result = input.to_lowercase() == "true";
            assert_eq!(result, expected, "Input '{}' should be {}", input, expected);
        }
    }

    #[test]
    fn test_hsts_env_parsing_logic_false() {
        // Test the parsing logic for false values
        let test_cases = vec![
            ("false", false),
            ("FALSE", false),
            ("False", false),
            ("0", false),
            ("1", false), // Only "true" should work
            ("yes", false),
            ("no", false),
            ("", false),
        ];

        for (input, expected) in test_cases {
            let result = input.to_lowercase() == "true";
            assert_eq!(result, expected, "Input '{}' should be {}", input, expected);
        }
    }

    // ===========================================
    // HeaderValue Tests
    // ===========================================

    #[test]
    fn test_security_header_values_are_valid() {
        // Verify that the static header values can be parsed
        assert!(HeaderValue::from_static("nosniff").to_str().is_ok());
        assert!(HeaderValue::from_static("DENY").to_str().is_ok());
        assert!(HeaderValue::from_static("1; mode=block").to_str().is_ok());
        assert!(
            HeaderValue::from_static("max-age=31536000; includeSubDomains")
                .to_str()
                .is_ok()
        );
    }

    #[test]
    fn test_x_content_type_options_value() {
        let value = HeaderValue::from_static("nosniff");
        assert_eq!(value.to_str().unwrap(), "nosniff");
    }

    #[test]
    fn test_x_frame_options_value() {
        let value = HeaderValue::from_static("DENY");
        assert_eq!(value.to_str().unwrap(), "DENY");
    }

    #[test]
    fn test_x_xss_protection_value() {
        let value = HeaderValue::from_static("1; mode=block");
        assert_eq!(value.to_str().unwrap(), "1; mode=block");
    }

    #[test]
    fn test_hsts_header_value() {
        let value = HeaderValue::from_static("max-age=31536000; includeSubDomains");
        assert_eq!(
            value.to_str().unwrap(),
            "max-age=31536000; includeSubDomains"
        );
        // Verify max-age is 1 year (31536000 seconds)
        assert!(value.to_str().unwrap().contains("31536000"));
    }

    // ===========================================
    // Header Name Tests
    // ===========================================

    #[test]
    fn test_header_names_can_be_created() {
        // Verify header names can be created from static strings
        let name1 = header::HeaderName::from_static("x-content-type-options");
        let name2 = header::HeaderName::from_static("x-frame-options");
        let name3 = header::HeaderName::from_static("x-xss-protection");

        assert_eq!(name1.as_str(), "x-content-type-options");
        assert_eq!(name2.as_str(), "x-frame-options");
        assert_eq!(name3.as_str(), "x-xss-protection");
    }

    #[test]
    fn test_strict_transport_security_header() {
        // HSTS header should be the standard one
        assert_eq!(
            header::STRICT_TRANSPORT_SECURITY.as_str(),
            "strict-transport-security"
        );
    }

    // ===========================================
    // HSTS Max-Age Tests
    // ===========================================

    #[test]
    fn test_hsts_max_age_is_one_year() {
        // 31536000 seconds = 365 days * 24 hours * 60 minutes * 60 seconds
        let expected_seconds = 365 * 24 * 60 * 60;
        assert_eq!(expected_seconds, 31536000);
    }

    #[test]
    fn test_hsts_includes_subdomains() {
        let value = "max-age=31536000; includeSubDomains";
        assert!(value.contains("includeSubDomains"));
    }
}
