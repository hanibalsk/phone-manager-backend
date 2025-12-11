//! System configuration repository.
//!
//! AP-9: System Configuration persistence operations

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    EmailTemplateEntity, FeatureFlagEntity, NotificationTemplateEntity, RateLimitConfigEntity,
    SystemSettingEntity,
};

/// Repository for system configuration operations.
#[derive(Clone)]
pub struct SystemConfigRepository {
    pool: PgPool,
}

impl SystemConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Feature Flags
    // ========================================================================

    /// Get all feature flags.
    pub async fn list_feature_flags(&self) -> Result<Vec<FeatureFlagEntity>, sqlx::Error> {
        sqlx::query_as::<_, FeatureFlagEntity>(
            r#"
            SELECT id, flag_key, enabled, description, category, updated_by, created_at, updated_at
            FROM feature_flags
            ORDER BY category, flag_key
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get a feature flag by key.
    pub async fn get_feature_flag(
        &self,
        flag_key: &str,
    ) -> Result<Option<FeatureFlagEntity>, sqlx::Error> {
        sqlx::query_as::<_, FeatureFlagEntity>(
            r#"
            SELECT id, flag_key, enabled, description, category, updated_by, created_at, updated_at
            FROM feature_flags
            WHERE flag_key = $1
            "#,
        )
        .bind(flag_key)
        .fetch_optional(&self.pool)
        .await
    }

    /// Get a feature flag by ID.
    pub async fn get_feature_flag_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<FeatureFlagEntity>, sqlx::Error> {
        sqlx::query_as::<_, FeatureFlagEntity>(
            r#"
            SELECT id, flag_key, enabled, description, category, updated_by, created_at, updated_at
            FROM feature_flags
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Update a feature flag.
    pub async fn update_feature_flag(
        &self,
        flag_key: &str,
        enabled: bool,
        updated_by: Uuid,
    ) -> Result<FeatureFlagEntity, sqlx::Error> {
        sqlx::query_as::<_, FeatureFlagEntity>(
            r#"
            UPDATE feature_flags
            SET enabled = $2, updated_by = $3, updated_at = NOW()
            WHERE flag_key = $1
            RETURNING id, flag_key, enabled, description, category, updated_by, created_at, updated_at
            "#,
        )
        .bind(flag_key)
        .bind(enabled)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
    }

    // ========================================================================
    // Rate Limits
    // ========================================================================

    /// Get all rate limit configurations.
    pub async fn list_rate_limits(&self) -> Result<Vec<RateLimitConfigEntity>, sqlx::Error> {
        sqlx::query_as::<_, RateLimitConfigEntity>(
            r#"
            SELECT id, limit_key, requests_per_period, period_seconds, description, updated_by, created_at, updated_at
            FROM rate_limit_configs
            ORDER BY limit_key
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get a rate limit configuration by key.
    pub async fn get_rate_limit(
        &self,
        limit_key: &str,
    ) -> Result<Option<RateLimitConfigEntity>, sqlx::Error> {
        sqlx::query_as::<_, RateLimitConfigEntity>(
            r#"
            SELECT id, limit_key, requests_per_period, period_seconds, description, updated_by, created_at, updated_at
            FROM rate_limit_configs
            WHERE limit_key = $1
            "#,
        )
        .bind(limit_key)
        .fetch_optional(&self.pool)
        .await
    }

    /// Update a rate limit configuration.
    pub async fn update_rate_limit(
        &self,
        limit_key: &str,
        requests_per_period: i32,
        updated_by: Uuid,
    ) -> Result<RateLimitConfigEntity, sqlx::Error> {
        sqlx::query_as::<_, RateLimitConfigEntity>(
            r#"
            UPDATE rate_limit_configs
            SET requests_per_period = $2, updated_by = $3, updated_at = NOW()
            WHERE limit_key = $1
            RETURNING id, limit_key, requests_per_period, period_seconds, description, updated_by, created_at, updated_at
            "#,
        )
        .bind(limit_key)
        .bind(requests_per_period)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
    }

    // ========================================================================
    // System Settings
    // ========================================================================

    /// Get all system settings.
    pub async fn list_system_settings(&self) -> Result<Vec<SystemSettingEntity>, sqlx::Error> {
        sqlx::query_as::<_, SystemSettingEntity>(
            r#"
            SELECT id, setting_key, setting_value, description, category, is_sensitive, updated_by, created_at, updated_at
            FROM system_settings
            ORDER BY category, setting_key
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get a system setting by key.
    pub async fn get_system_setting(
        &self,
        setting_key: &str,
    ) -> Result<Option<SystemSettingEntity>, sqlx::Error> {
        sqlx::query_as::<_, SystemSettingEntity>(
            r#"
            SELECT id, setting_key, setting_value, description, category, is_sensitive, updated_by, created_at, updated_at
            FROM system_settings
            WHERE setting_key = $1
            "#,
        )
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await
    }

    /// Upsert a system setting.
    pub async fn upsert_system_setting(
        &self,
        setting_key: &str,
        setting_value: serde_json::Value,
        description: Option<&str>,
        category: &str,
        is_sensitive: bool,
        updated_by: Uuid,
    ) -> Result<SystemSettingEntity, sqlx::Error> {
        sqlx::query_as::<_, SystemSettingEntity>(
            r#"
            INSERT INTO system_settings (setting_key, setting_value, description, category, is_sensitive, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (setting_key) DO UPDATE
            SET setting_value = $2, description = COALESCE($3, system_settings.description),
                is_sensitive = $5, updated_by = $6, updated_at = NOW()
            RETURNING id, setting_key, setting_value, description, category, is_sensitive, updated_by, created_at, updated_at
            "#,
        )
        .bind(setting_key)
        .bind(setting_value)
        .bind(description)
        .bind(category)
        .bind(is_sensitive)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
    }

    // ========================================================================
    // Notification Templates
    // ========================================================================

    /// Get all notification templates.
    pub async fn list_notification_templates(
        &self,
    ) -> Result<Vec<NotificationTemplateEntity>, sqlx::Error> {
        sqlx::query_as::<_, NotificationTemplateEntity>(
            r#"
            SELECT id, template_key, template_type, title_template, body_template, data_schema, is_active, updated_by, created_at, updated_at
            FROM notification_templates
            ORDER BY template_type, template_key
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get a notification template by ID.
    pub async fn get_notification_template(
        &self,
        id: Uuid,
    ) -> Result<Option<NotificationTemplateEntity>, sqlx::Error> {
        sqlx::query_as::<_, NotificationTemplateEntity>(
            r#"
            SELECT id, template_key, template_type, title_template, body_template, data_schema, is_active, updated_by, created_at, updated_at
            FROM notification_templates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Update a notification template.
    pub async fn update_notification_template(
        &self,
        id: Uuid,
        title_template: Option<&str>,
        body_template: Option<&str>,
        data_schema: Option<serde_json::Value>,
        is_active: Option<bool>,
        updated_by: Uuid,
    ) -> Result<NotificationTemplateEntity, sqlx::Error> {
        sqlx::query_as::<_, NotificationTemplateEntity>(
            r#"
            UPDATE notification_templates
            SET
                title_template = COALESCE($2, title_template),
                body_template = COALESCE($3, body_template),
                data_schema = COALESCE($4, data_schema),
                is_active = COALESCE($5, is_active),
                updated_by = $6,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, template_key, template_type, title_template, body_template, data_schema, is_active, updated_by, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(title_template)
        .bind(body_template)
        .bind(data_schema)
        .bind(is_active)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
    }

    // ========================================================================
    // Email Templates
    // ========================================================================

    /// Get all email templates.
    pub async fn list_email_templates(&self) -> Result<Vec<EmailTemplateEntity>, sqlx::Error> {
        sqlx::query_as::<_, EmailTemplateEntity>(
            r#"
            SELECT id, template_key, subject_template, body_html_template, body_text_template, data_schema, is_active, updated_by, created_at, updated_at
            FROM email_templates
            ORDER BY template_key
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Get an email template by ID.
    pub async fn get_email_template(
        &self,
        id: Uuid,
    ) -> Result<Option<EmailTemplateEntity>, sqlx::Error> {
        sqlx::query_as::<_, EmailTemplateEntity>(
            r#"
            SELECT id, template_key, subject_template, body_html_template, body_text_template, data_schema, is_active, updated_by, created_at, updated_at
            FROM email_templates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Update an email template.
    pub async fn update_email_template(
        &self,
        id: Uuid,
        subject_template: Option<&str>,
        body_html_template: Option<&str>,
        body_text_template: Option<&str>,
        data_schema: Option<serde_json::Value>,
        is_active: Option<bool>,
        updated_by: Uuid,
    ) -> Result<EmailTemplateEntity, sqlx::Error> {
        sqlx::query_as::<_, EmailTemplateEntity>(
            r#"
            UPDATE email_templates
            SET
                subject_template = COALESCE($2, subject_template),
                body_html_template = COALESCE($3, body_html_template),
                body_text_template = COALESCE($4, body_text_template),
                data_schema = COALESCE($5, data_schema),
                is_active = COALESCE($6, is_active),
                updated_by = $7,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, template_key, subject_template, body_html_template, body_text_template, data_schema, is_active, updated_by, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(subject_template)
        .bind(body_html_template)
        .bind(body_text_template)
        .bind(data_schema)
        .bind(is_active)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
    }
}
