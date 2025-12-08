//! Public configuration endpoint for feature flags.
//!
//! Exposes server configuration that clients need to know about,
//! such as enabled features and authentication modes.

use axum::{extract::State, Json};
use serde::Serialize;

use crate::app::AppState;

/// Public configuration response.
/// Contains feature flags and auth configuration that clients can use
/// to adjust their UI and behavior.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PublicConfigResponse {
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Feature toggles
    pub features: FeaturesConfig,
}

/// Authentication configuration visible to clients.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthConfig {
    /// Whether email/password registration is enabled
    pub registration_enabled: bool,
    /// Whether invite-only mode is active (registration requires invite token)
    pub invite_only: bool,
    /// Whether OAuth-only mode is active (no password-based auth)
    pub oauth_only: bool,
    /// Whether Google OAuth is configured
    pub google_enabled: bool,
    /// Whether Apple OAuth is configured
    pub apple_enabled: bool,
}

/// Feature flags visible to clients.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeaturesConfig {
    /// Whether geofences feature is enabled
    pub geofences: bool,
    /// Whether proximity alerts feature is enabled
    pub proximity_alerts: bool,
    /// Whether webhooks feature is enabled
    pub webhooks: bool,
    /// Whether movement tracking (trips) feature is enabled
    pub movement_tracking: bool,
    /// Whether B2B/organization features are enabled
    pub b2b: bool,
    /// Whether geofence events feature is enabled
    pub geofence_events: bool,
}

/// GET /api/v1/config/public
///
/// Returns public configuration including feature flags and auth modes.
/// This endpoint is unauthenticated so clients can check configuration
/// before attempting to authenticate.
pub async fn get_public_config(State(state): State<AppState>) -> Json<PublicConfigResponse> {
    let config = &state.config;

    let response = PublicConfigResponse {
        auth: AuthConfig {
            registration_enabled: config.auth_toggles.registration_enabled,
            invite_only: config.auth_toggles.invite_only,
            oauth_only: config.auth_toggles.oauth_only,
            google_enabled: !config.oauth.google_client_id.is_empty(),
            apple_enabled: !config.oauth.apple_client_id.is_empty(),
        },
        features: FeaturesConfig {
            geofences: config.features.geofences_enabled,
            proximity_alerts: config.features.proximity_alerts_enabled,
            webhooks: config.features.webhooks_enabled,
            movement_tracking: config.features.movement_tracking_enabled,
            b2b: config.features.b2b_enabled,
            geofence_events: config.features.geofence_events_enabled,
        },
    };

    Json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_config_response_serialization() {
        let response = PublicConfigResponse {
            auth: AuthConfig {
                registration_enabled: true,
                invite_only: false,
                oauth_only: false,
                google_enabled: true,
                apple_enabled: false,
            },
            features: FeaturesConfig {
                geofences: true,
                proximity_alerts: true,
                webhooks: true,
                movement_tracking: true,
                b2b: false,
                geofence_events: true,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"registration_enabled\":true"));
        assert!(json.contains("\"google_enabled\":true"));
        assert!(json.contains("\"geofences\":true"));
        assert!(json.contains("\"b2b\":false"));
    }

    #[test]
    fn test_auth_config_all_disabled() {
        let config = AuthConfig {
            registration_enabled: false,
            invite_only: true,
            oauth_only: true,
            google_enabled: false,
            apple_enabled: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"registration_enabled\":false"));
        assert!(json.contains("\"invite_only\":true"));
        assert!(json.contains("\"oauth_only\":true"));
    }

    #[test]
    fn test_features_config_mixed() {
        let config = FeaturesConfig {
            geofences: true,
            proximity_alerts: false,
            webhooks: true,
            movement_tracking: false,
            b2b: true,
            geofence_events: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"geofences\":true"));
        assert!(json.contains("\"proximity_alerts\":false"));
        assert!(json.contains("\"movement_tracking\":false"));
    }
}
