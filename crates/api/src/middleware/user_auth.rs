//! User JWT authentication middleware.
//!
//! Provides middleware for requiring JWT-based user authentication on routes.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::app::AppState;
use crate::config::JwtAuthConfig;
use shared::jwt::JwtConfig;

/// Authenticated user information extracted from JWT.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used by handlers after middleware inserts into extensions
pub struct UserAuth {
    /// User ID from the JWT subject claim.
    pub user_id: Uuid,
    /// JWT ID (jti) for session tracking.
    pub jti: String,
}

impl UserAuth {
    /// Validates an access token and returns user authentication info.
    pub fn validate(jwt_config: &JwtConfig, token: &str) -> Result<Self, String> {
        // Validate the access token
        let claims = jwt_config
            .validate_access_token(token)
            .map_err(|e| format!("Invalid token: {}", e))?;

        // Parse user ID from claims
        let user_id =
            Uuid::parse_str(&claims.sub).map_err(|_| "Invalid user ID in token".to_string())?;

        Ok(UserAuth {
            user_id,
            jti: claims.jti,
        })
    }

    /// Creates a JwtConfig from JwtAuthConfig.
    pub fn create_jwt_config(config: &JwtAuthConfig) -> Result<JwtConfig, String> {
        JwtConfig::with_leeway(
            &config.private_key,
            &config.public_key,
            config.access_token_expiry_secs,
            config.refresh_token_expiry_secs,
            config.leeway_secs,
        )
        .map_err(|e| format!("Failed to initialize JWT config: {}", e))
    }
}

/// Middleware that requires JWT user authentication.
///
/// This middleware validates the Bearer token in the Authorization header
/// and rejects requests without a valid JWT. Authenticated user information
/// is stored in request extensions for use by downstream handlers.
#[allow(dead_code)] // Will be used when user-authenticated routes are added
pub async fn require_user_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Extract Bearer token from Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return unauthorized_response("Missing or invalid Authorization header");
        }
    };

    // Create JWT config
    let jwt_config = match UserAuth::create_jwt_config(&state.config.jwt) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to create JWT config: {}", e);
            return internal_error_response("Authentication service unavailable");
        }
    };

    // Validate the token
    match UserAuth::validate(&jwt_config, token) {
        Ok(auth) => {
            // Store authentication info in request extensions
            req.extensions_mut().insert(auth);
            next.run(req).await
        }
        Err(e) => {
            tracing::debug!("JWT validation failed: {}", e);
            unauthorized_response("Invalid or expired token")
        }
    }
}

/// Middleware that optionally validates JWT user authentication.
///
/// This middleware attempts to validate the Bearer token if present,
/// but allows the request to proceed even without authentication.
/// Authenticated user information (if valid) is stored in request extensions.
#[allow(dead_code)] // Will be used for optional auth routes
pub async fn optional_user_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Try to extract Bearer token from Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    if let Some(header) = auth_header {
        if let Some(token) = header.strip_prefix("Bearer ") {
            // Try to create JWT config and validate
            if let Ok(jwt_config) = UserAuth::create_jwt_config(&state.config.jwt) {
                if let Ok(auth) = UserAuth::validate(&jwt_config, token) {
                    req.extensions_mut().insert(auth);
                }
            }
        }
    }

    next.run(req).await
}

/// Helper to create unauthorized response.
#[allow(dead_code)] // Used by middleware functions
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

/// Helper to create internal error response.
#[allow(dead_code)] // Used by middleware functions
fn internal_error_response(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "internal_error",
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
    fn test_unauthorized_response_missing_token() {
        let response = unauthorized_response("Missing or invalid Authorization header");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_unauthorized_response_invalid_token() {
        let response = unauthorized_response("Invalid or expired token");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_internal_error_response() {
        let response = internal_error_response("Authentication service unavailable");
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_user_auth_struct() {
        let auth = UserAuth {
            user_id: Uuid::new_v4(),
            jti: "test_jti".to_string(),
        };
        assert!(!auth.jti.is_empty());
    }

    #[test]
    fn test_user_auth_clone() {
        let auth = UserAuth {
            user_id: Uuid::new_v4(),
            jti: "test_jti".to_string(),
        };
        let cloned = auth.clone();
        assert_eq!(auth.user_id, cloned.user_id);
        assert_eq!(auth.jti, cloned.jti);
    }

    #[test]
    fn test_user_auth_debug() {
        let auth = UserAuth {
            user_id: Uuid::new_v4(),
            jti: "test_jti".to_string(),
        };
        let debug_str = format!("{:?}", auth);
        assert!(debug_str.contains("UserAuth"));
        assert!(debug_str.contains("user_id"));
        assert!(debug_str.contains("jti"));
    }
}
