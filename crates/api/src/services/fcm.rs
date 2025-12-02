//! Firebase Cloud Messaging (FCM) notification service.
//!
//! Implements the NotificationService trait using FCM HTTP v1 API for
//! sending push notifications to Android/iOS devices.

use std::sync::RwLock;
use std::time::{Duration, Instant};

use chrono::Utc;
use domain::services::{
    NotificationResult, NotificationService, SettingsChangedPayload, UnlockRequestResponsePayload,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::FcmConfig;

/// FCM notification service using Firebase Cloud Messaging HTTP v1 API.
pub struct FcmNotificationService {
    client: Client,
    config: FcmConfig,
    /// Service account credentials parsed from JSON.
    credentials: ServiceAccountCredentials,
    /// Cached access token with expiry tracking.
    token_cache: RwLock<Option<CachedToken>>,
}

/// Cached OAuth2 access token.
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

/// Google service account credentials structure.
#[derive(Debug, Clone, Deserialize)]
struct ServiceAccountCredentials {
    /// Service account email.
    client_email: String,
    /// Private key in PEM format.
    private_key: String,
    /// Token URI for OAuth2 exchange.
    token_uri: String,
}

/// JWT claims for Google OAuth2 service account authentication.
#[derive(Debug, Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

/// Google OAuth2 token response.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    #[allow(dead_code)]
    token_type: String,
}

/// FCM v1 API message structure.
#[derive(Debug, Serialize)]
struct FcmMessage {
    message: MessagePayload,
}

#[derive(Debug, Serialize)]
struct MessagePayload {
    token: String,
    data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    android: Option<AndroidConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    apns: Option<ApnsConfig>,
}

#[derive(Debug, Serialize)]
struct AndroidConfig {
    priority: String,
}

#[derive(Debug, Serialize)]
struct ApnsConfig {
    headers: ApnsHeaders,
}

#[derive(Debug, Serialize)]
struct ApnsHeaders {
    #[serde(rename = "apns-priority")]
    priority: String,
}

/// FCM API error response (for future detailed error handling).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FcmErrorResponse {
    error: FcmErrorDetails,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FcmErrorDetails {
    message: String,
    status: String,
}

/// Error type for FCM operations.
#[derive(Debug, thiserror::Error)]
pub enum FcmError {
    #[error("Failed to parse credentials: {0}")]
    CredentialsError(String),

    #[error("Failed to create JWT: {0}")]
    JwtError(String),

    #[error("Failed to get access token: {0}")]
    TokenError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("FCM API error: {0}")]
    ApiError(String),

    #[error("Invalid FCM token")]
    InvalidToken,

    #[error("FCM is not enabled")]
    NotEnabled,
}

impl FcmNotificationService {
    /// Create a new FCM notification service.
    ///
    /// # Arguments
    /// * `config` - FCM configuration with project ID and credentials
    ///
    /// # Errors
    /// Returns an error if credentials cannot be parsed.
    pub fn new(config: FcmConfig) -> Result<Self, FcmError> {
        if !config.enabled {
            return Err(FcmError::NotEnabled);
        }

        // Parse credentials from JSON (either file path or inline JSON)
        let credentials = Self::load_credentials(&config.credentials)?;

        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| FcmError::HttpError(e))?;

        Ok(Self {
            client,
            config,
            credentials,
            token_cache: RwLock::new(None),
        })
    }

    /// Load service account credentials from JSON string or file path.
    fn load_credentials(credentials_source: &str) -> Result<ServiceAccountCredentials, FcmError> {
        // Try parsing as JSON first
        if credentials_source.trim().starts_with('{') {
            serde_json::from_str(credentials_source)
                .map_err(|e| FcmError::CredentialsError(format!("Invalid JSON: {}", e)))
        } else {
            // Treat as file path
            let content = std::fs::read_to_string(credentials_source).map_err(|e| {
                FcmError::CredentialsError(format!("Failed to read credentials file: {}", e))
            })?;
            serde_json::from_str(&content)
                .map_err(|e| FcmError::CredentialsError(format!("Invalid credentials JSON: {}", e)))
        }
    }

    /// Get a valid OAuth2 access token, refreshing if necessary.
    async fn get_access_token(&self) -> Result<String, FcmError> {
        // Check cache first
        {
            let cache = self.token_cache.read().unwrap();
            if let Some(ref token) = *cache {
                // Return cached token if still valid (with 60s buffer)
                if token.expires_at > Instant::now() + Duration::from_secs(60) {
                    return Ok(token.access_token.clone());
                }
            }
        }

        // Token expired or not cached, get a new one
        let (access_token, expires_at) = self.fetch_access_token().await?;

        // Cache the new token
        {
            let mut cache = self.token_cache.write().unwrap();
            *cache = Some(CachedToken {
                access_token: access_token.clone(),
                expires_at,
            });
        }

        Ok(access_token)
    }

    /// Fetch a new OAuth2 access token from Google.
    /// Returns (access_token, expires_at).
    async fn fetch_access_token(&self) -> Result<(String, Instant), FcmError> {
        let now = Utc::now().timestamp();

        // Create JWT claims
        let claims = JwtClaims {
            iss: self.credentials.client_email.clone(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
            aud: self.credentials.token_uri.clone(),
            iat: now,
            exp: now + 3600, // 1 hour
        };

        // Create and sign JWT
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let encoding_key =
            jsonwebtoken::EncodingKey::from_rsa_pem(self.credentials.private_key.as_bytes())
                .map_err(|e| FcmError::JwtError(format!("Invalid private key: {}", e)))?;

        let jwt = jsonwebtoken::encode(&header, &claims, &encoding_key)
            .map_err(|e| FcmError::JwtError(format!("Failed to create JWT: {}", e)))?;

        // Exchange JWT for access token
        let response = self
            .client
            .post(&self.credentials.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(FcmError::TokenError(format!(
                "Token exchange failed: {}",
                error_text
            )));
        }

        let token_response: TokenResponse = response.json().await?;
        let expires_at = Instant::now() + Duration::from_secs(token_response.expires_in);

        Ok((token_response.access_token, expires_at))
    }

    /// Send a data message to a device via FCM.
    async fn send_message(
        &self,
        fcm_token: &str,
        data: serde_json::Value,
    ) -> Result<(), FcmError> {
        let access_token = self.get_access_token().await?;

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.config.project_id
        );

        let message = FcmMessage {
            message: MessagePayload {
                token: fcm_token.to_string(),
                data,
                android: if self.config.high_priority {
                    Some(AndroidConfig {
                        priority: "high".to_string(),
                    })
                } else {
                    None
                },
                apns: if self.config.high_priority {
                    Some(ApnsConfig {
                        headers: ApnsHeaders {
                            priority: "10".to_string(),
                        },
                    })
                } else {
                    None
                },
            },
        };

        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                // Exponential backoff: 100ms, 200ms, 400ms, etc.
                tokio::time::sleep(Duration::from_millis(100 * (1 << (attempt - 1)))).await;
            }

            let response = self
                .client
                .post(&url)
                .bearer_auth(&access_token)
                .json(&message)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        tracing::debug!(
                            fcm_token = %fcm_token,
                            attempt = %attempt,
                            "FCM message sent successfully"
                        );
                        return Ok(());
                    }

                    // Check for unrecoverable errors
                    let status = resp.status();
                    if status.as_u16() == 404 || status.as_u16() == 400 {
                        // Invalid token - don't retry
                        let error_text = resp.text().await.unwrap_or_default();
                        if error_text.contains("UNREGISTERED")
                            || error_text.contains("INVALID_ARGUMENT")
                        {
                            return Err(FcmError::InvalidToken);
                        }
                        return Err(FcmError::ApiError(error_text));
                    }

                    // Retry on 5xx errors
                    if status.is_server_error() {
                        let error_text = resp.text().await.unwrap_or_default();
                        last_error = Some(FcmError::ApiError(error_text));
                        continue;
                    }

                    // Other client errors
                    let error_text = resp.text().await.unwrap_or_default();
                    return Err(FcmError::ApiError(error_text));
                }
                Err(e) => {
                    last_error = Some(FcmError::HttpError(e));
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| FcmError::ApiError("Unknown error".to_string())))
    }
}

#[async_trait::async_trait]
impl NotificationService for FcmNotificationService {
    async fn send_settings_changed(
        &self,
        fcm_token: &str,
        payload: SettingsChangedPayload,
    ) -> NotificationResult {
        let data = match serde_json::to_value(&payload) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "Failed to serialize notification payload");
                return NotificationResult::Failed(format!("Serialization error: {}", e));
            }
        };

        match self.send_message(fcm_token, data).await {
            Ok(()) => {
                tracing::info!(
                    fcm_token = %fcm_token,
                    device_id = %payload.device_id,
                    changes_count = %payload.changes.len(),
                    "Settings changed notification sent"
                );
                NotificationResult::Sent
            }
            Err(FcmError::InvalidToken) => {
                tracing::warn!(
                    fcm_token = %fcm_token,
                    device_id = %payload.device_id,
                    "Invalid FCM token - device should re-register"
                );
                NotificationResult::NoToken
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    fcm_token = %fcm_token,
                    device_id = %payload.device_id,
                    "Failed to send settings changed notification"
                );
                NotificationResult::Failed(e.to_string())
            }
        }
    }

    async fn send_unlock_request_response(
        &self,
        fcm_token: &str,
        payload: UnlockRequestResponsePayload,
    ) -> NotificationResult {
        let data = match serde_json::to_value(&payload) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "Failed to serialize notification payload");
                return NotificationResult::Failed(format!("Serialization error: {}", e));
            }
        };

        match self.send_message(fcm_token, data).await {
            Ok(()) => {
                tracing::info!(
                    fcm_token = %fcm_token,
                    request_id = %payload.request_id,
                    status = %payload.status,
                    "Unlock request response notification sent"
                );
                NotificationResult::Sent
            }
            Err(FcmError::InvalidToken) => {
                tracing::warn!(
                    fcm_token = %fcm_token,
                    request_id = %payload.request_id,
                    "Invalid FCM token - device should re-register"
                );
                NotificationResult::NoToken
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    fcm_token = %fcm_token,
                    request_id = %payload.request_id,
                    "Failed to send unlock request response notification"
                );
                NotificationResult::Failed(e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcm_not_enabled_error() {
        let config = FcmConfig {
            enabled: false,
            ..Default::default()
        };
        let result = FcmNotificationService::new(config);
        assert!(matches!(result, Err(FcmError::NotEnabled)));
    }

    #[test]
    fn test_load_credentials_invalid_json() {
        let result = FcmNotificationService::load_credentials("not valid json");
        assert!(matches!(result, Err(FcmError::CredentialsError(_))));
    }

    #[test]
    fn test_load_credentials_inline_json() {
        let json = r#"{
            "client_email": "test@project.iam.gserviceaccount.com",
            "private_key": "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----\n",
            "token_uri": "https://oauth2.googleapis.com/token"
        }"#;

        let result = FcmNotificationService::load_credentials(json);
        assert!(result.is_ok());
        let creds = result.unwrap();
        assert_eq!(creds.client_email, "test@project.iam.gserviceaccount.com");
    }

    #[test]
    fn test_fcm_message_serialization() {
        let message = FcmMessage {
            message: MessagePayload {
                token: "test_token".to_string(),
                data: serde_json::json!({"key": "value"}),
                android: Some(AndroidConfig {
                    priority: "high".to_string(),
                }),
                apns: Some(ApnsConfig {
                    headers: ApnsHeaders {
                        priority: "10".to_string(),
                    },
                }),
            },
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("test_token"));
        assert!(json.contains("high"));
    }

    #[test]
    fn test_jwt_claims_serialization() {
        let claims = JwtClaims {
            iss: "test@example.com".to_string(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            iat: 1234567890,
            exp: 1234571490,
        };

        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("firebase.messaging"));
    }
}
