//! Request tracing middleware.
//!
//! Provides request ID extraction and generation for distributed tracing.

use axum::{
    body::Body,
    http::{header::HeaderName, Extensions, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Header name for request ID.
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Request ID stored in request extensions.
#[derive(Debug, Clone)]
pub struct RequestId(#[allow(dead_code)] pub String);

/// Middleware that extracts or generates a request ID.
///
/// If the `X-Request-ID` header is present, uses that value.
/// Otherwise, generates a new UUID v4.
///
/// The request ID is:
/// 1. Stored in request extensions for downstream handlers
/// 2. Added to the response headers
/// 3. Used in tracing spans for log correlation
pub async fn trace_id(mut req: Request<Body>, next: Next) -> Response {
    // Extract or generate request ID
    let request_id = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store in request extensions for handlers
    req.extensions_mut().insert(RequestId(request_id.clone()));

    // Create a tracing span with the request ID
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %req.method(),
        path = %req.uri().path(),
    );

    // Execute the request within the span
    let _guard = span.enter();
    let start = std::time::Instant::now();

    let mut response = next.run(req).await;

    // Log request completion
    let duration_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();

    tracing::info!(
        request_id = %request_id,
        status = status,
        duration_ms = duration_ms,
        "Request completed"
    );

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), header_value);
    }

    response
}

/// Extracts the request ID from request extensions.
///
/// Returns the request ID if present, or a placeholder if not.
#[allow(dead_code)] // Used by handlers to access request ID
pub fn get_request_id(extensions: &Extensions) -> String {
    extensions
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_struct() {
        let id = RequestId("test-id-123".to_string());
        assert_eq!(id.0, "test-id-123");
    }

    #[test]
    fn test_request_id_struct_clone() {
        let id = RequestId("test-id".to_string());
        let cloned = id.clone();
        assert_eq!(cloned.0, "test-id");
    }

    #[test]
    fn test_request_id_struct_debug() {
        let id = RequestId("debug-test".to_string());
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("debug-test"));
    }

    #[test]
    fn test_get_request_id_missing() {
        let extensions = Extensions::new();
        assert_eq!(get_request_id(&extensions), "unknown");
    }

    #[test]
    fn test_get_request_id_present() {
        let mut extensions = Extensions::new();
        extensions.insert(RequestId("my-request-id".to_string()));
        assert_eq!(get_request_id(&extensions), "my-request-id");
    }

    #[test]
    fn test_get_request_id_uuid_format() {
        let mut extensions = Extensions::new();
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        extensions.insert(RequestId(uuid_str.to_string()));
        assert_eq!(get_request_id(&extensions), uuid_str);
    }

    #[test]
    fn test_request_id_header_constant() {
        assert_eq!(REQUEST_ID_HEADER, "X-Request-ID");
    }

    #[test]
    fn test_request_id_with_special_characters() {
        let mut extensions = Extensions::new();
        // Valid header value characters
        extensions.insert(RequestId("req-123_abc.xyz".to_string()));
        assert_eq!(get_request_id(&extensions), "req-123_abc.xyz");
    }

    #[test]
    fn test_request_id_empty_string() {
        let mut extensions = Extensions::new();
        extensions.insert(RequestId(String::new()));
        assert_eq!(get_request_id(&extensions), "");
    }
}
