//! System configuration domain models for admin API.
//!
//! AP-9: System Configuration endpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// System settings response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SystemSettingsResponse {
    /// Server configuration
    pub server: ServerSettingsInfo,
    /// Database configuration (sensitive info redacted)
    pub database: DatabaseSettingsInfo,
    /// Logging configuration
    pub logging: LoggingSettingsInfo,
    /// Security configuration
    pub security: SecuritySettingsInfo,
    /// Limits configuration
    pub limits: LimitsSettingsInfo,
    /// Map matching configuration
    pub map_matching: MapMatchingSettingsInfo,
    /// Email configuration (credentials redacted)
    pub email: EmailSettingsInfo,
    /// FCM configuration (credentials redacted)
    pub fcm: FcmSettingsInfo,
    /// Frontend configuration
    pub frontend: FrontendSettingsInfo,
}

/// Server settings info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ServerSettingsInfo {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub max_body_size: usize,
    pub app_base_url: String,
}

/// Database settings info (connection string redacted).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DatabaseSettingsInfo {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

/// Logging settings info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LoggingSettingsInfo {
    pub level: String,
    pub format: String,
}

/// Security settings info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SecuritySettingsInfo {
    pub cors_origins: Vec<String>,
    pub rate_limit_per_minute: u32,
    pub export_rate_limit_per_hour: u32,
    pub forgot_password_rate_limit_per_hour: u32,
    pub request_verification_rate_limit_per_hour: u32,
}

/// Limits settings info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LimitsSettingsInfo {
    pub max_devices_per_group: usize,
    pub max_batch_size: usize,
    pub location_retention_days: u32,
    pub max_display_name_length: usize,
    pub max_group_id_length: usize,
    pub max_webhooks_per_device: Option<u32>,
    pub warning_threshold_percent: u32,
}

/// Map matching settings info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MapMatchingSettingsInfo {
    pub provider: String,
    pub enabled: bool,
    pub timeout_ms: u64,
    pub rate_limit_per_minute: u32,
    pub circuit_breaker_failures: u32,
    pub circuit_breaker_reset_secs: u64,
}

/// Email settings info (credentials redacted).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EmailSettingsInfo {
    pub enabled: bool,
    pub provider: String,
    pub sender_email: String,
    pub sender_name: String,
    pub template_style: String,
}

/// FCM settings info (credentials redacted).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FcmSettingsInfo {
    pub enabled: bool,
    pub project_id: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub high_priority: bool,
}

/// Frontend settings info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FrontendSettingsInfo {
    pub enabled: bool,
    pub base_dir: String,
    pub staging_hostname: String,
    pub production_hostname: String,
    pub default_environment: String,
    pub immutable_cache_max_age: u32,
    pub mutable_cache_max_age: u32,
}

/// Feature flags response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeatureFlagsResponse {
    pub features: FeatureFlagsInfo,
    pub auth: AuthTogglesInfo,
}

/// Feature flags info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeatureFlagsInfo {
    pub geofences_enabled: bool,
    pub proximity_alerts_enabled: bool,
    pub webhooks_enabled: bool,
    pub movement_tracking_enabled: bool,
    pub b2b_enabled: bool,
    pub geofence_events_enabled: bool,
}

/// Auth toggles info.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthTogglesInfo {
    pub registration_enabled: bool,
    pub invite_only: bool,
    pub oauth_only: bool,
    pub google_enabled: bool,
    pub apple_enabled: bool,
}

/// Rate limits response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RateLimitsResponse {
    /// General API rate limit per minute
    pub rate_limit_per_minute: u32,
    /// Export rate limit per hour per organization
    pub export_rate_limit_per_hour: u32,
    /// Forgot password rate limit per hour per IP
    pub forgot_password_rate_limit_per_hour: u32,
    /// Request verification rate limit per hour per IP
    pub request_verification_rate_limit_per_hour: u32,
    /// Map matching rate limit per minute
    pub map_matching_rate_limit_per_minute: u32,
}

/// Maintenance mode status response.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MaintenanceModeResponse {
    /// Whether maintenance mode is enabled
    pub enabled: bool,
    /// Optional message to display during maintenance
    pub message: Option<String>,
    /// When maintenance mode was enabled (if enabled)
    pub enabled_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Estimated end time (if provided)
    pub estimated_end: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to toggle maintenance mode.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ToggleMaintenanceModeRequest {
    /// Whether to enable or disable maintenance mode
    pub enabled: bool,
    /// Optional message to display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Optional estimated end time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_end: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// Request Models for Mutation APIs (FR-9.2 through FR-9.6)
// ============================================================================

/// Request to update a feature flag.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateFeatureFlagRequest {
    /// Whether the flag is enabled
    pub enabled: bool,
}

/// Response after updating a feature flag.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeatureFlagResponse {
    pub id: Uuid,
    pub flag_key: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub category: String,
    pub updated_at: DateTime<Utc>,
}

/// Request to update rate limits.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateRateLimitsRequest {
    /// General API rate limit per minute
    #[validate(range(
        min = 1,
        max = 10000,
        message = "Rate limit must be between 1 and 10000"
    ))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_minute: Option<u32>,
    /// Export rate limit per hour
    #[validate(range(
        min = 1,
        max = 1000,
        message = "Export rate limit must be between 1 and 1000"
    ))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_rate_limit_per_hour: Option<u32>,
    /// Forgot password rate limit per hour
    #[validate(range(
        min = 1,
        max = 100,
        message = "Forgot password rate limit must be between 1 and 100"
    ))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forgot_password_rate_limit_per_hour: Option<u32>,
    /// Request verification rate limit per hour
    #[validate(range(
        min = 1,
        max = 100,
        message = "Request verification rate limit must be between 1 and 100"
    ))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_verification_rate_limit_per_hour: Option<u32>,
    /// Map matching rate limit per minute
    #[validate(range(
        min = 1,
        max = 1000,
        message = "Map matching rate limit must be between 1 and 1000"
    ))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map_matching_rate_limit_per_minute: Option<u32>,
}

/// Rate limit configuration item.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RateLimitConfigItem {
    pub id: Uuid,
    pub limit_key: String,
    pub requests_per_period: i32,
    pub period_seconds: i32,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Notification Templates (FR-9.3)
// ============================================================================

/// Notification template.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NotificationTemplate {
    pub id: Uuid,
    pub template_key: String,
    pub template_type: String,
    pub title_template: String,
    pub body_template: String,
    pub data_schema: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to update a notification template.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateNotificationTemplateRequest {
    /// Title template with placeholders
    #[validate(length(min = 1, max = 500, message = "Title must be 1-500 characters"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_template: Option<String>,
    /// Body template with placeholders
    #[validate(length(min = 1, max = 5000, message = "Body must be 1-5000 characters"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_template: Option<String>,
    /// JSON schema for template variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_schema: Option<serde_json::Value>,
    /// Whether the template is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Response for list notification templates.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NotificationTemplatesResponse {
    pub templates: Vec<NotificationTemplate>,
    pub total: i64,
}

// ============================================================================
// Email Templates (FR-9.6)
// ============================================================================

/// Email template.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EmailTemplate {
    pub id: Uuid,
    pub template_key: String,
    pub subject_template: String,
    pub body_html_template: String,
    pub body_text_template: Option<String>,
    pub data_schema: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to update an email template.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateEmailTemplateRequest {
    /// Subject template with placeholders
    #[validate(length(min = 1, max = 500, message = "Subject must be 1-500 characters"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_template: Option<String>,
    /// HTML body template with placeholders
    #[validate(length(min = 1, max = 50000, message = "HTML body must be 1-50000 characters"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html_template: Option<String>,
    /// Plain text body template with placeholders
    #[validate(length(max = 50000, message = "Text body must be at most 50000 characters"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text_template: Option<String>,
    /// JSON schema for template variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_schema: Option<serde_json::Value>,
    /// Whether the template is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Response for list email templates.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EmailTemplatesResponse {
    pub templates: Vec<EmailTemplate>,
    pub total: i64,
}

// ============================================================================
// System Settings Update (FR-9.2)
// ============================================================================

/// Request to update system settings.
/// Only non-sensitive settings can be updated at runtime.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateSystemSettingsRequest {
    /// Logging level (trace, debug, info, warn, error)
    #[validate(length(min = 1, max = 20))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging_level: Option<String>,
    /// CORS origins (comma-separated or array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors_origins: Option<Vec<String>>,
    /// Max devices per group
    #[validate(range(min = 1, max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_devices_per_group: Option<u32>,
    /// Max batch size for location uploads
    #[validate(range(min = 1, max = 1000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_batch_size: Option<u32>,
    /// Location retention in days
    #[validate(range(min = 1, max = 365))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_retention_days: Option<u32>,
    /// Warning threshold percent for limits
    #[validate(range(min = 50, max = 99))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_threshold_percent: Option<u32>,
}

/// System setting item (key-value).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SystemSettingItem {
    pub id: Uuid,
    pub setting_key: String,
    pub setting_value: serde_json::Value,
    pub description: Option<String>,
    pub category: String,
    pub is_sensitive: bool,
    pub updated_at: DateTime<Utc>,
}

/// Response after updating system settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateSystemSettingsResponse {
    pub updated: Vec<String>,
    pub settings: SystemSettingsResponse,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flags_response_serialization() {
        let response = FeatureFlagsResponse {
            features: FeatureFlagsInfo {
                geofences_enabled: true,
                proximity_alerts_enabled: true,
                webhooks_enabled: false,
                movement_tracking_enabled: true,
                b2b_enabled: true,
                geofence_events_enabled: true,
            },
            auth: AuthTogglesInfo {
                registration_enabled: true,
                invite_only: false,
                oauth_only: false,
                google_enabled: true,
                apple_enabled: false,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"geofences_enabled\":true"));
        assert!(json.contains("\"webhooks_enabled\":false"));
        assert!(json.contains("\"registration_enabled\":true"));
    }

    #[test]
    fn test_rate_limits_response_serialization() {
        let response = RateLimitsResponse {
            rate_limit_per_minute: 100,
            export_rate_limit_per_hour: 10,
            forgot_password_rate_limit_per_hour: 5,
            request_verification_rate_limit_per_hour: 3,
            map_matching_rate_limit_per_minute: 30,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"rate_limit_per_minute\":100"));
        assert!(json.contains("\"export_rate_limit_per_hour\":10"));
    }

    #[test]
    fn test_maintenance_mode_response_serialization() {
        let response = MaintenanceModeResponse {
            enabled: true,
            message: Some("System maintenance in progress".to_string()),
            enabled_at: None,
            estimated_end: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"message\":\"System maintenance in progress\""));
    }

    #[test]
    fn test_toggle_maintenance_mode_request_deserialization() {
        let json = r#"{"enabled":true,"message":"Scheduled maintenance"}"#;
        let request: ToggleMaintenanceModeRequest = serde_json::from_str(json).unwrap();
        assert!(request.enabled);
        assert_eq!(request.message, Some("Scheduled maintenance".to_string()));
    }
}
