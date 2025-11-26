//! Authentication middleware.
//!
//! Provides middleware for requiring API key authentication on routes.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::app::AppState;
use crate::extractors::api_key::ApiKeyAuth;

/// Middleware that requires API key authentication.
///
/// This middleware validates the `X-API-Key` header and rejects requests
/// without a valid API key. Authenticated key information is stored in
/// request extensions for use by downstream handlers.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Extract API key from header
    let api_key = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok());

    let api_key = match api_key {
        Some(key) => key.to_string(),
        None => {
            return unauthorized_response("Invalid or missing API key");
        }
    };

    // Validate the API key
    match ApiKeyAuth::validate(&state.pool, &api_key).await {
        Ok(auth) => {
            // Store authentication info in request extensions
            req.extensions_mut().insert(auth);
            next.run(req).await
        }
        Err(err) => err.into_response(),
    }
}

/// Middleware that optionally validates API key authentication.
///
/// This middleware attempts to validate the `X-API-Key` header if present,
/// but allows the request to proceed even without authentication.
/// Authenticated key information (if valid) is stored in request extensions.
#[allow(dead_code)] // Will be used in future stories
pub async fn optional_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Try to extract API key from header
    if let Some(api_key) = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok()) {
        // Validate the API key if present
        if let Ok(auth) = ApiKeyAuth::validate(&state.pool, api_key).await {
            req.extensions_mut().insert(auth);
        }
    }

    next.run(req).await
}

/// Middleware for admin-only routes.
///
/// This middleware requires API key authentication AND the key must have
/// admin privileges.
#[allow(dead_code)] // Will be used in Story 4.7 (Admin Operations API)
pub async fn require_admin(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Extract API key from header
    let api_key = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok());

    let api_key = match api_key {
        Some(key) => key.to_string(),
        None => {
            return unauthorized_response("Invalid or missing API key");
        }
    };

    // Validate the API key
    match ApiKeyAuth::validate(&state.pool, &api_key).await {
        Ok(auth) => {
            if !auth.is_admin {
                return forbidden_response("Admin access required");
            }
            req.extensions_mut().insert(auth);
            next.run(req).await
        }
        Err(err) => err.into_response(),
    }
}

/// Helper to create unauthorized response.
fn unauthorized_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "unauthorized",
            "message": message
        })),
    )
        .into_response()
}

/// Helper to create forbidden response.
#[allow(dead_code)] // Will be used in Story 4.7 (Admin Operations API)
fn forbidden_response(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": "forbidden",
            "message": message
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unauthorized_response() {
        let response = unauthorized_response("Test message");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_unauthorized_response_custom_message() {
        let response = unauthorized_response("Invalid or missing API key");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_unauthorized_response_empty_message() {
        let response = unauthorized_response("");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_forbidden_response() {
        let response = forbidden_response("Test message");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_forbidden_response_admin_message() {
        let response = forbidden_response("Admin access required");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_forbidden_response_empty_message() {
        let response = forbidden_response("");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
