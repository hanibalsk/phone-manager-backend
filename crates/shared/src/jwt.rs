//! JWT token utilities using RS256 algorithm.
//!
//! This module provides JWT token generation and validation using RS256
//! (RSA-SHA256) asymmetric signing for secure authentication.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Error type for JWT operations.
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Failed to encode token: {0}")]
    EncodingError(String),

    #[error("Failed to decode token: {0}")]
    DecodingError(String),

    #[error("Token has expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

/// JWT token claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// JWT ID (unique token identifier for revocation)
    pub jti: String,
    /// Token type (access or refresh)
    pub token_type: TokenType,
}

/// Type of JWT token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// Configuration for JWT token generation and validation.
#[derive(Clone)]
pub struct JwtConfig {
    /// RSA private key in PEM format for signing tokens
    encoding_key: EncodingKey,
    /// RSA public key in PEM format for validating tokens
    decoding_key: DecodingKey,
    /// Access token expiration in seconds (default: 900 = 15 minutes)
    pub access_token_expiry_secs: i64,
    /// Refresh token expiration in seconds (default: 604800 = 7 days)
    pub refresh_token_expiry_secs: i64,
    /// Leeway in seconds for clock skew tolerance (default: 30)
    pub leeway_secs: u64,
}

impl std::fmt::Debug for JwtConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtConfig")
            .field("access_token_expiry_secs", &self.access_token_expiry_secs)
            .field("refresh_token_expiry_secs", &self.refresh_token_expiry_secs)
            .field("leeway_secs", &self.leeway_secs)
            .field("encoding_key", &"[REDACTED]")
            .field("decoding_key", &"[REDACTED]")
            .finish()
    }
}

/// Default leeway in seconds for clock skew tolerance
pub const DEFAULT_LEEWAY_SECS: u64 = 30;

impl JwtConfig {
    /// Creates a new JwtConfig from RSA key pair in PEM format.
    ///
    /// # Arguments
    /// * `private_key_pem` - RSA private key in PEM format
    /// * `public_key_pem` - RSA public key in PEM format
    /// * `access_token_expiry_secs` - Access token expiration in seconds
    /// * `refresh_token_expiry_secs` - Refresh token expiration in seconds
    pub fn new(
        private_key_pem: &str,
        public_key_pem: &str,
        access_token_expiry_secs: i64,
        refresh_token_expiry_secs: i64,
    ) -> Result<Self, JwtError> {
        Self::with_leeway(
            private_key_pem,
            public_key_pem,
            access_token_expiry_secs,
            refresh_token_expiry_secs,
            DEFAULT_LEEWAY_SECS,
        )
    }

    /// Creates a new JwtConfig from RSA key pair in PEM format with custom leeway.
    ///
    /// # Arguments
    /// * `private_key_pem` - RSA private key in PEM format
    /// * `public_key_pem` - RSA public key in PEM format
    /// * `access_token_expiry_secs` - Access token expiration in seconds
    /// * `refresh_token_expiry_secs` - Refresh token expiration in seconds
    /// * `leeway_secs` - Leeway in seconds for clock skew tolerance
    pub fn with_leeway(
        private_key_pem: &str,
        public_key_pem: &str,
        access_token_expiry_secs: i64,
        refresh_token_expiry_secs: i64,
        leeway_secs: u64,
    ) -> Result<Self, JwtError> {
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(|e| JwtError::InvalidKey(format!("Invalid private key: {}", e)))?;

        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| JwtError::InvalidKey(format!("Invalid public key: {}", e)))?;

        Ok(Self {
            encoding_key,
            decoding_key,
            access_token_expiry_secs,
            refresh_token_expiry_secs,
            leeway_secs,
        })
    }

    /// Creates a JwtConfig for testing with HS256 symmetric key.
    /// DO NOT use in production - only for tests.
    #[cfg(test)]
    pub fn new_for_testing(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_expiry_secs: 900,
            refresh_token_expiry_secs: 604800,
            leeway_secs: 0, // Strict for testing - no leeway
        }
    }

    /// Generates an access token for the given user ID.
    pub fn generate_access_token(&self, user_id: Uuid) -> Result<(String, String), JwtError> {
        self.generate_token(user_id, TokenType::Access, self.access_token_expiry_secs)
    }

    /// Generates a refresh token for the given user ID.
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<(String, String), JwtError> {
        self.generate_token(user_id, TokenType::Refresh, self.refresh_token_expiry_secs)
    }

    /// Generates a token with the specified type and expiration.
    fn generate_token(
        &self,
        user_id: Uuid,
        token_type: TokenType,
        expiry_secs: i64,
    ) -> Result<(String, String), JwtError> {
        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();
        let exp = (now + Duration::seconds(expiry_secs)).timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
            iat: now.timestamp(),
            jti: jti.clone(),
            token_type,
        };

        // Use RS256 for production, but tests may use HS256
        let header = Header::new(self.algorithm());

        let token = encode(&header, &claims, &self.encoding_key)
            .map_err(|e| JwtError::EncodingError(e.to_string()))?;

        Ok((token, jti))
    }

    /// Validates a token and returns its claims.
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(self.algorithm());
        validation.validate_exp = true;
        // Set leeway for clock skew tolerance (default: 30 seconds)
        // This allows for minor clock differences between client and server
        validation.leeway = self.leeway_secs;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidToken
                | jsonwebtoken::errors::ErrorKind::InvalidSignature => JwtError::InvalidToken,
                _ => JwtError::DecodingError(e.to_string()),
            }
        })?;

        Ok(token_data.claims)
    }

    /// Validates an access token specifically.
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.validate_token(token)?;
        if claims.token_type != TokenType::Access {
            return Err(JwtError::InvalidToken);
        }
        Ok(claims)
    }

    /// Validates a refresh token specifically.
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.validate_token(token)?;
        if claims.token_type != TokenType::Refresh {
            return Err(JwtError::InvalidToken);
        }
        Ok(claims)
    }

    /// Returns the algorithm used by this config.
    /// Tests use HS256, production uses RS256.
    fn algorithm(&self) -> Algorithm {
        // The encoding key type determines the algorithm
        // For tests with secret key, we need HS256
        // For RSA keys, we use RS256
        #[cfg(test)]
        {
            Algorithm::HS256
        }
        #[cfg(not(test))]
        {
            Algorithm::RS256
        }
    }
}

/// Extracts user ID from validated claims.
pub fn extract_user_id(claims: &Claims) -> Result<Uuid, JwtError> {
    Uuid::parse_str(&claims.sub).map_err(|_| JwtError::InvalidToken)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;

    fn create_test_config() -> JwtConfig {
        JwtConfig::new_for_testing("test_secret_key_for_jwt_testing_12345")
    }

    fn create_test_config_short_expiry() -> JwtConfig {
        let mut config = create_test_config();
        config.access_token_expiry_secs = 1; // 1 second for testing expiration
        config
    }

    #[test]
    fn test_generate_access_token() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let (token, jti) = config.generate_access_token(user_id).unwrap();

        assert!(!token.is_empty());
        assert!(!jti.is_empty());
        assert!(token.contains('.'), "JWT should have dots separating parts");
    }

    #[test]
    fn test_generate_refresh_token() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let (token, jti) = config.generate_refresh_token(user_id).unwrap();

        assert!(!token.is_empty());
        assert!(!jti.is_empty());
    }

    #[test]
    fn test_validate_access_token() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let (token, jti) = config.generate_access_token(user_id).unwrap();
        let claims = config.validate_access_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.jti, jti);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_validate_refresh_token() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let (token, jti) = config.generate_refresh_token(user_id).unwrap();
        let claims = config.validate_refresh_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.jti, jti);
        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_access_token_rejected_as_refresh() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let (token, _) = config.generate_access_token(user_id).unwrap();
        let result = config.validate_refresh_token(&token);

        assert!(matches!(result, Err(JwtError::InvalidToken)));
    }

    #[test]
    fn test_refresh_token_rejected_as_access() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let (token, _) = config.generate_refresh_token(user_id).unwrap();
        let result = config.validate_access_token(&token);

        assert!(matches!(result, Err(JwtError::InvalidToken)));
    }

    #[test]
    fn test_expired_token() {
        let config = create_test_config_short_expiry();
        let user_id = Uuid::new_v4();

        let (token, _) = config.generate_access_token(user_id).unwrap();

        // Wait for token to expire
        sleep(StdDuration::from_secs(2));

        let result = config.validate_access_token(&token);
        assert!(
            matches!(result, Err(JwtError::TokenExpired)),
            "Expected TokenExpired, got: {:?}",
            result
        );
    }

    #[test]
    fn test_invalid_token() {
        let config = create_test_config();
        let result = config.validate_token("invalid.token.here");

        assert!(matches!(
            result,
            Err(JwtError::InvalidToken) | Err(JwtError::DecodingError(_))
        ));
    }

    #[test]
    fn test_malformed_token() {
        let config = create_test_config();
        let result = config.validate_token("not_a_jwt");

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_user_id() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let (token, _) = config.generate_access_token(user_id).unwrap();
        let claims = config.validate_access_token(&token).unwrap();
        let extracted_id = extract_user_id(&claims).unwrap();

        assert_eq!(extracted_id, user_id);
    }

    #[test]
    fn test_unique_jti_per_token() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let (_, jti1) = config.generate_access_token(user_id).unwrap();
        let (_, jti2) = config.generate_access_token(user_id).unwrap();

        assert_ne!(jti1, jti2, "Each token should have unique jti");
    }

    #[test]
    fn test_claims_timestamps() {
        let config = create_test_config();
        let user_id = Uuid::new_v4();

        let before = Utc::now().timestamp();
        let (token, _) = config.generate_access_token(user_id).unwrap();
        let after = Utc::now().timestamp();

        let claims = config.validate_access_token(&token).unwrap();

        assert!(claims.iat >= before && claims.iat <= after);
        assert!(claims.exp > claims.iat);
        assert_eq!(
            claims.exp - claims.iat,
            config.access_token_expiry_secs as i64
        );
    }

    #[test]
    fn test_jwt_error_display() {
        assert!(format!("{}", JwtError::TokenExpired).contains("expired"));
        assert!(format!("{}", JwtError::InvalidToken).contains("Invalid"));
        assert!(format!("{}", JwtError::EncodingError("test".to_string())).contains("encode"));
        assert!(format!("{}", JwtError::DecodingError("test".to_string())).contains("decode"));
    }

    #[test]
    fn test_token_type_serialization() {
        let access = TokenType::Access;
        let refresh = TokenType::Refresh;

        assert_eq!(serde_json::to_string(&access).unwrap(), "\"access\"");
        assert_eq!(serde_json::to_string(&refresh).unwrap(), "\"refresh\"");
    }
}
