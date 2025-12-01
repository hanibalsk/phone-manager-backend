//! User JWT authentication extractor.
//!
//! Provides an Axum extractor for validating JWT tokens from requests.

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::middleware::user_auth::UserAuth as UserAuthData;

/// Authenticated user information from JWT.
///
/// This extractor validates the Bearer token in the Authorization header
/// and provides access to the authenticated user's details.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in user-authenticated route handlers
pub struct UserAuth {
    /// User ID from the JWT subject claim.
    pub user_id: Uuid,
    /// JWT ID (jti) for session tracking.
    pub jti: String,
}

impl From<UserAuthData> for UserAuth {
    fn from(data: UserAuthData) -> Self {
        Self {
            user_id: data.user_id,
            jti: data.jti,
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for UserAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // First, check if auth info was already inserted by middleware
        if let Some(auth) = parts.extensions.get::<UserAuthData>() {
            return Ok(auth.clone().into());
        }

        // Otherwise, extract and validate the token directly
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized(
                "Invalid Authorization header format".to_string(),
            ));
        }

        let token = &auth_header[7..];

        // Create JWT config
        let jwt_config = UserAuthData::create_jwt_config(&state.config.jwt)
            .map_err(|e| ApiError::Internal(e))?;

        // Validate the token
        let auth_data = UserAuthData::validate(&jwt_config, token)
            .map_err(|_| ApiError::Unauthorized("Invalid or expired token".to_string()))?;

        Ok(auth_data.into())
    }
}

/// Optional user JWT authentication.
///
/// This extractor allows routes to optionally check for authentication
/// without rejecting unauthenticated requests.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in optional-auth route handlers
pub struct OptionalUserAuth(pub Option<UserAuth>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalUserAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // First, check if auth info was already inserted by middleware
        if let Some(auth) = parts.extensions.get::<UserAuthData>() {
            return Ok(OptionalUserAuth(Some(auth.clone().into())));
        }

        // Try to extract Bearer token
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = &header[7..];

                // Try to create JWT config and validate
                if let Ok(jwt_config) = UserAuthData::create_jwt_config(&state.config.jwt) {
                    if let Ok(auth_data) = UserAuthData::validate(&jwt_config, token) {
                        return Ok(OptionalUserAuth(Some(auth_data.into())));
                    }
                }
                Ok(OptionalUserAuth(None))
            }
            _ => Ok(OptionalUserAuth(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_optional_user_auth_none() {
        let auth = OptionalUserAuth(None);
        assert!(auth.0.is_none());
    }

    #[test]
    fn test_optional_user_auth_some() {
        let auth = OptionalUserAuth(Some(UserAuth {
            user_id: Uuid::new_v4(),
            jti: "test_jti".to_string(),
        }));
        assert!(auth.0.is_some());
    }

    #[test]
    fn test_optional_user_auth_clone() {
        let auth = OptionalUserAuth(Some(UserAuth {
            user_id: Uuid::new_v4(),
            jti: "test_jti".to_string(),
        }));
        let cloned = auth.clone();
        assert!(cloned.0.is_some());
    }

    #[test]
    fn test_optional_user_auth_debug() {
        let auth = OptionalUserAuth(None);
        let debug_str = format!("{:?}", auth);
        assert!(debug_str.contains("OptionalUserAuth"));
    }

    #[test]
    fn test_user_auth_from_data() {
        let data = UserAuthData {
            user_id: Uuid::new_v4(),
            jti: "test_jti".to_string(),
        };
        let auth: UserAuth = data.into();
        assert!(!auth.jti.is_empty());
    }
}
