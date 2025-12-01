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

/// Request body for user login.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// User's email address
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    /// User's password
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,

    /// Optional device ID making the request
    #[allow(dead_code)] // Used in future story for device linking
    pub device_id: Option<String>,

    /// Optional device name
    #[allow(dead_code)] // Used in future story for device linking
    pub device_name: Option<String>,
}

/// Request body for OAuth sign-in.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct OAuthLoginRequest {
    /// OAuth provider (google or apple)
    #[validate(length(min = 1, message = "Provider is required"))]
    pub provider: String,

    /// ID token from the OAuth provider
    #[validate(length(min = 1, message = "ID token is required"))]
    pub id_token: String,

    /// Optional device ID making the request
    #[allow(dead_code)] // Used in future story for device linking
    pub device_id: Option<String>,

    /// Optional device name
    #[allow(dead_code)] // Used in future story for device linking
    pub device_name: Option<String>,
}

/// Response body for successful login.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub user: UserResponse,
    pub tokens: TokensResponse,
}

/// Login with email and password.
///
/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // Login user
    let result = auth_service
        .login(&request.email, &request.password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => {
                ApiError::Unauthorized("Invalid email or password".to_string())
            }
            AuthError::UserDisabled => ApiError::Forbidden("User account is disabled".to_string()),
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            AuthError::PasswordError(e) => {
                // Log the actual error but return generic message
                tracing::error!("Password verification error: {}", e);
                ApiError::Unauthorized("Invalid email or password".to_string())
            }
            AuthError::TokenError(e) => ApiError::Internal(format!("Token error: {}", e)),
            _ => ApiError::Internal(e.to_string()),
        })?;

    // Build response
    let response = LoginResponse {
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
    };

    Ok(Json(response))
}

/// OAuth sign-in with Google or Apple.
///
/// POST /api/v1/auth/oauth
pub async fn oauth_login(
    State(state): State<AppState>,
    Json(request): Json<OAuthLoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // OAuth login
    let result = auth_service
        .oauth_login(&request.provider, &request.id_token)
        .await
        .map_err(|e| match e {
            AuthError::InvalidOAuthToken => {
                ApiError::Unauthorized("Invalid or expired OAuth token".to_string())
            }
            AuthError::UnsupportedOAuthProvider => {
                ApiError::Validation("Unsupported OAuth provider. Use 'google' or 'apple'.".to_string())
            }
            AuthError::OAuthProviderError(msg) => {
                tracing::error!("OAuth provider error: {}", msg);
                ApiError::Internal("OAuth provider error".to_string())
            }
            AuthError::UserDisabled => ApiError::Forbidden("User account is disabled".to_string()),
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            _ => ApiError::Internal(e.to_string()),
        })?;

    // Determine auth provider for response
    let auth_provider = request.provider.to_lowercase();

    // Build response
    let response = LoginResponse {
        user: UserResponse {
            id: result.user_id.to_string(),
            email: result.email,
            display_name: result.display_name,
            avatar_url: None,
            email_verified: result.email_verified,
            auth_provider,
            organization_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        tokens: TokensResponse {
            access_token: result.access_token,
            refresh_token: result.refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: result.access_token_expires_in,
        },
    };

    Ok(Json(response))
}

/// Request body for token refresh.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    /// The refresh token to use
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// Response body for successful token refresh.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub tokens: TokensResponse,
}

/// Refresh access token using a valid refresh token.
///
/// POST /api/v1/auth/refresh
pub async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // Refresh tokens
    let result = auth_service
        .refresh(&request.refresh_token)
        .await
        .map_err(|e| match e {
            AuthError::InvalidRefreshToken => {
                ApiError::Unauthorized("Invalid or expired refresh token".to_string())
            }
            AuthError::SessionNotFound => {
                ApiError::Unauthorized("Session not found or revoked".to_string())
            }
            AuthError::UserNotFound => ApiError::Unauthorized("User not found".to_string()),
            AuthError::UserDisabled => ApiError::Forbidden("User account is disabled".to_string()),
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            AuthError::TokenError(e) => {
                tracing::error!("Token error during refresh: {}", e);
                ApiError::Unauthorized("Invalid or expired refresh token".to_string())
            }
            _ => ApiError::Internal(e.to_string()),
        })?;

    // Build response
    let response = RefreshResponse {
        tokens: TokensResponse {
            access_token: result.access_token,
            refresh_token: result.refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: result.expires_in,
        },
    };

    Ok(Json(response))
}

/// Request body for logout.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    /// The refresh token to invalidate
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,

    /// If true, invalidate all sessions for the user
    #[serde(default)]
    pub all_devices: bool,
}

/// Logout and invalidate tokens.
///
/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<StatusCode, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // Logout - invalidate the session
    auth_service
        .logout(&request.refresh_token, request.all_devices)
        .await
        .map_err(|e| match e {
            AuthError::InvalidRefreshToken => {
                ApiError::Unauthorized("Invalid or expired refresh token".to_string())
            }
            AuthError::TokenError(e) => {
                tracing::error!("Token error during logout: {}", e);
                ApiError::Unauthorized("Invalid or expired refresh token".to_string())
            }
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Request body for forgot password.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ForgotPasswordRequest {
    /// User's email address
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// Response body for forgot password (always success for security).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgotPasswordResponse {
    pub message: String,
}

/// Request password reset - initiates the password reset flow.
///
/// POST /api/v1/auth/forgot-password
///
/// Always returns 200 to prevent email enumeration attacks.
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // Request password reset (silently ignores non-existent emails)
    auth_service
        .forgot_password(&request.email)
        .await
        .map_err(|e| match e {
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            _ => ApiError::Internal(e.to_string()),
        })?;

    // Always return success to prevent email enumeration
    Ok(Json(ForgotPasswordResponse {
        message: "If your email is registered, you will receive a password reset link.".to_string(),
    }))
}

/// Request body for reset password.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    /// The password reset token from the email
    #[validate(length(min = 1, message = "Reset token is required"))]
    pub token: String,

    /// The new password
    #[validate(length(min = 1, message = "New password is required"))]
    pub new_password: String,
}

/// Response body for reset password.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordResponse {
    pub message: String,
}

/// Reset password using a valid reset token.
///
/// POST /api/v1/auth/reset-password
pub async fn reset_password(
    State(state): State<AppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // Reset password
    auth_service
        .reset_password(&request.token, &request.new_password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidResetToken => {
                ApiError::Validation("Invalid or expired reset token".to_string())
            }
            AuthError::WeakPassword(msg) => ApiError::Validation(msg),
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            AuthError::PasswordError(e) => ApiError::Internal(format!("Password error: {}", e)),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(Json(ResetPasswordResponse {
        message: "Password has been reset successfully. Please log in with your new password."
            .to_string(),
    }))
}

/// Response body for request verification.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestVerificationResponse {
    pub message: String,
}

/// Request a new email verification token.
///
/// POST /api/v1/auth/request-verification
///
/// Requires authentication (JWT bearer token).
pub async fn request_verification(
    State(state): State<AppState>,
    user_auth: crate::extractors::UserAuth,
) -> Result<Json<RequestVerificationResponse>, ApiError> {
    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // Request verification
    auth_service
        .request_email_verification(user_auth.user_id)
        .await
        .map_err(|e| match e {
            AuthError::UserNotFound => ApiError::NotFound("User not found".to_string()),
            AuthError::EmailAlreadyVerified => {
                ApiError::Conflict("Email is already verified".to_string())
            }
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(Json(RequestVerificationResponse {
        message: "Verification email has been sent.".to_string(),
    }))
}

/// Request body for verify email.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailRequest {
    /// The email verification token
    #[validate(length(min = 1, message = "Verification token is required"))]
    pub token: String,
}

/// Response body for verify email.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailResponse {
    pub message: String,
    pub email_verified: bool,
}

/// Verify email using a verification token.
///
/// POST /api/v1/auth/verify-email
pub async fn verify_email(
    State(state): State<AppState>,
    Json(request): Json<VerifyEmailRequest>,
) -> Result<Json<VerifyEmailResponse>, ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Create auth service
    let auth_service = AuthService::new(state.pool.clone(), &state.config.jwt)
        .map_err(|e| ApiError::Internal(format!("Failed to initialize auth service: {}", e)))?;

    // Verify email
    auth_service
        .verify_email(&request.token)
        .await
        .map_err(|e| match e {
            AuthError::InvalidVerificationToken => {
                ApiError::Validation("Invalid or expired verification token".to_string())
            }
            AuthError::EmailAlreadyVerified => {
                ApiError::Conflict("Email is already verified".to_string())
            }
            AuthError::DatabaseError(db_err) => ApiError::from(db_err),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(Json(VerifyEmailResponse {
        message: "Email has been verified successfully.".to_string(),
        email_verified: true,
    }))
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

    #[test]
    fn test_login_request_validation() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "SecureP@ss1".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_login_request_invalid_email() {
        let request = LoginRequest {
            email: "not-an-email".to_string(),
            password: "SecureP@ss1".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_login_request_empty_password() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_refresh_request_validation() {
        let request = RefreshRequest {
            refresh_token: "some.refresh.token".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_refresh_request_empty_token() {
        let request = RefreshRequest {
            refresh_token: "".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_logout_request_validation() {
        let request = LogoutRequest {
            refresh_token: "some.refresh.token".to_string(),
            all_devices: false,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_logout_request_with_all_devices() {
        let request = LogoutRequest {
            refresh_token: "some.refresh.token".to_string(),
            all_devices: true,
        };

        assert!(request.validate().is_ok());
        assert!(request.all_devices);
    }

    #[test]
    fn test_logout_request_empty_token() {
        let request = LogoutRequest {
            refresh_token: "".to_string(),
            all_devices: false,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_logout_request_default_all_devices() {
        // Test that all_devices defaults to false when not provided
        let json = r#"{"refreshToken": "some.token"}"#;
        let request: LogoutRequest = serde_json::from_str(json).unwrap();
        assert!(!request.all_devices);
    }

    #[test]
    fn test_forgot_password_request_validation() {
        let request = ForgotPasswordRequest {
            email: "test@example.com".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_forgot_password_request_invalid_email() {
        let request = ForgotPasswordRequest {
            email: "not-an-email".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_forgot_password_request_empty_email() {
        let request = ForgotPasswordRequest {
            email: "".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_reset_password_request_validation() {
        let request = ResetPasswordRequest {
            token: "abc123".to_string(),
            new_password: "SecureP@ss1".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_reset_password_request_empty_token() {
        let request = ResetPasswordRequest {
            token: "".to_string(),
            new_password: "SecureP@ss1".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_reset_password_request_empty_password() {
        let request = ResetPasswordRequest {
            token: "abc123".to_string(),
            new_password: "".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_verify_email_request_validation() {
        let request = VerifyEmailRequest {
            token: "abc123".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_verify_email_request_empty_token() {
        let request = VerifyEmailRequest {
            token: "".to_string(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_verify_email_response_serialization() {
        let response = VerifyEmailResponse {
            message: "Email verified".to_string(),
            email_verified: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("emailVerified"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_request_verification_response_serialization() {
        let response = RequestVerificationResponse {
            message: "Verification sent".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("message"));
        assert!(json.contains("Verification sent"));
    }

    #[test]
    fn test_oauth_login_request_validation() {
        let request = OAuthLoginRequest {
            provider: "google".to_string(),
            id_token: "some.id.token".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_oauth_login_request_empty_provider() {
        let request = OAuthLoginRequest {
            provider: "".to_string(),
            id_token: "some.id.token".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_oauth_login_request_empty_token() {
        let request = OAuthLoginRequest {
            provider: "google".to_string(),
            id_token: "".to_string(),
            device_id: None,
            device_name: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_oauth_login_request_with_device() {
        let request = OAuthLoginRequest {
            provider: "apple".to_string(),
            id_token: "some.id.token".to_string(),
            device_id: Some("device-123".to_string()),
            device_name: Some("My iPhone".to_string()),
        };

        assert!(request.validate().is_ok());
        assert_eq!(request.device_id, Some("device-123".to_string()));
    }

    #[test]
    fn test_oauth_login_request_deserialization() {
        let json = r#"{"provider": "google", "idToken": "abc.def.ghi"}"#;
        let request: OAuthLoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.provider, "google");
        assert_eq!(request.id_token, "abc.def.ghi");
        assert!(request.device_id.is_none());
    }
}
