//! Authentication routes for user registration, login, and token management.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::services::auth::{AuthError, AuthService};

/// Request body for user registration.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// User's email address
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    /// User's password (min 8 chars, 1 upper, 1 lower, 1 digit)
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,

    /// User's display name
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: String,

    /// Optional device ID to link after registration
    #[allow(dead_code)] // Used in future story for device linking
    pub device_id: Option<String>,

    /// Device name (required if device_id provided)
    #[allow(dead_code)] // Used in future story for device linking
    pub device_name: Option<String>,
}

/// User information in response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub auth_provider: String,
    pub organization_id: Option<String>,
    pub created_at: String,
}

/// Token information in response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Response body for successful registration.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub user: UserResponse,
    pub tokens: TokensResponse,
    pub device_linked: bool,
    pub requires_email_verification: bool,
}

/// Register a new user with email and password.
///
/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // Register user
    let result = auth_service
        .register(&request.email, &request.password, &request.display_name)
        .await
        .map_err(|e| match e {
            AuthError::EmailAlreadyExists => {
                ApiError::Conflict("Email already registered".to_string())
            }
            AuthError::WeakPassword(msg) => ApiError::Validation(msg),
            AuthError::InvalidEmail => ApiError::Validation("Invalid email format".to_string()),
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            AuthError::PasswordError(e) => ApiError::Internal(format!("Password error: {}", e)),
            AuthError::TokenError(e) => ApiError::Internal(format!("Token error: {}", e)),
            _ => ApiError::Internal(e.to_string()),
        })?;

    // Build response
    let response = RegisterResponse {
        user: UserResponse {
            id: result.user_id.to_string(),
            email: result.email,
            display_name: result.display_name,
            avatar_url: None,
            email_verified: result.email_verified,
            auth_provider: "email".to_string(),
            organization_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        tokens: TokensResponse {
            access_token: result.access_token,
            refresh_token: result.refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: result.access_token_expires_in,
        },
        device_linked: false, // Device linking will be implemented later
        requires_email_verification: true,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_validation() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "SecureP@ss1".to_string(),
            display_name: "Test User".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_register_request_invalid_email() {
        let request = RegisterRequest {
            email: "not-an-email".to_string(),
            password: "SecureP@ss1".to_string(),
            display_name: "Test User".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_request_empty_password() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
            display_name: "Test User".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_request_empty_display_name() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "SecureP@ss1".to_string(),
            display_name: "".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_register_request_long_display_name() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "SecureP@ss1".to_string(),
            display_name: "A".repeat(101),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_err());
    }
}
