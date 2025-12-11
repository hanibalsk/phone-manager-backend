//! System configuration database entities.
//!
//! AP-9: System Configuration persistence layer

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Feature flag entity.
#[derive(Debug, Clone, FromRow)]
pub struct FeatureFlagEntity {
    pub id: Uuid,
    pub flag_key: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub category: String,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Rate limit configuration entity.
#[derive(Debug, Clone, FromRow)]
pub struct RateLimitConfigEntity {
    pub id: Uuid,
    pub limit_key: String,
    pub requests_per_period: i32,
    pub period_seconds: i32,
    pub description: Option<String>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// System setting entity.
#[derive(Debug, Clone, FromRow)]
pub struct SystemSettingEntity {
    pub id: Uuid,
    pub setting_key: String,
    pub setting_value: serde_json::Value,
    pub description: Option<String>,
    pub category: String,
    pub is_sensitive: bool,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Notification template entity.
#[derive(Debug, Clone, FromRow)]
pub struct NotificationTemplateEntity {
    pub id: Uuid,
    pub template_key: String,
    pub template_type: String,
    pub title_template: String,
    pub body_template: String,
    pub data_schema: Option<serde_json::Value>,
    pub is_active: bool,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Email template entity.
#[derive(Debug, Clone, FromRow)]
pub struct EmailTemplateEntity {
    pub id: Uuid,
    pub template_key: String,
    pub subject_template: String,
    pub body_html_template: String,
    pub body_text_template: Option<String>,
    pub data_schema: Option<serde_json::Value>,
    pub is_active: bool,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
