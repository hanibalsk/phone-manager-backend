//! System configuration route handlers.
//!
//! AP-9: System Configuration endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use std::sync::RwLock;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::middleware::system_rbac::SystemRoleAuth;

use domain::models::{
    AuthTogglesInfo, DatabaseSettingsInfo, EmailSettingsInfo, EmailTemplate, EmailTemplatesResponse,
    FcmSettingsInfo, FeatureFlagResponse, FeatureFlagsInfo, FeatureFlagsResponse,
    FrontendSettingsInfo, LimitsSettingsInfo, LoggingSettingsInfo, MaintenanceModeResponse,
    MapMatchingSettingsInfo, NotificationTemplate, NotificationTemplatesResponse,
    RateLimitConfigItem, RateLimitsResponse, SecuritySettingsInfo, ServerSettingsInfo,
    SystemSettingsResponse, ToggleMaintenanceModeRequest, UpdateEmailTemplateRequest,
    UpdateFeatureFlagRequest, UpdateNotificationTemplateRequest, UpdateRateLimitsRequest,
};
use persistence::repositories::SystemConfigRepository;

/// In-memory maintenance mode state.
/// In a production system, this would be stored in a database or distributed cache.
static MAINTENANCE_MODE: RwLock<Option<MaintenanceState>> = RwLock::new(None);

struct MaintenanceState {
    enabled: bool,
    message: Option<String>,
    enabled_at: chrono::DateTime<chrono::Utc>,
    estimated_end: Option<chrono::DateTime<chrono::Utc>>,
}

/// Create system configuration routes.
///
/// These routes require super_admin role for management operations.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings", get(get_system_settings))
        .route("/feature-flags", get(get_feature_flags))
        .route("/feature-flags/{flag_id}", put(update_feature_flag))
        .route(
            "/rate-limits",
            get(get_rate_limits).put(update_rate_limits),
        )
        .route(
            "/maintenance",
            get(get_maintenance_mode).post(toggle_maintenance_mode),
        )
        .route("/templates", get(list_notification_templates))
        .route("/templates/{template_id}", put(update_notification_template))
        .route("/email-templates", get(list_email_templates))
        .route("/email-templates/{template_id}", put(update_email_template))
}

/// Get system settings.
///
/// GET /api/admin/v1/system/settings
///
/// Returns current system configuration with sensitive values redacted.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn get_system_settings(
    State(state): State<AppState>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can view system settings
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let config = &state.config;

    let response = SystemSettingsResponse {
        server: ServerSettingsInfo {
            host: config.server.host.clone(),
            port: config.server.port,
            request_timeout_secs: config.server.request_timeout_secs,
            max_body_size: config.server.max_body_size,
            app_base_url: config.server.app_base_url.clone(),
        },
        database: DatabaseSettingsInfo {
            max_connections: config.database.max_connections,
            min_connections: config.database.min_connections,
            connect_timeout_secs: config.database.connect_timeout_secs,
            idle_timeout_secs: config.database.idle_timeout_secs,
        },
        logging: LoggingSettingsInfo {
            level: config.logging.level.clone(),
            format: config.logging.format.clone(),
        },
        security: SecuritySettingsInfo {
            cors_origins: config.security.cors_origins.clone(),
            rate_limit_per_minute: config.security.rate_limit_per_minute,
            export_rate_limit_per_hour: config.security.export_rate_limit_per_hour,
            forgot_password_rate_limit_per_hour: config
                .security
                .forgot_password_rate_limit_per_hour,
            request_verification_rate_limit_per_hour: config
                .security
                .request_verification_rate_limit_per_hour,
        },
        limits: LimitsSettingsInfo {
            max_devices_per_group: config.limits.max_devices_per_group,
            max_batch_size: config.limits.max_batch_size,
            location_retention_days: config.limits.location_retention_days,
            max_display_name_length: config.limits.max_display_name_length,
            max_group_id_length: config.limits.max_group_id_length,
            max_webhooks_per_device: config.limits.max_webhooks_per_device,
            warning_threshold_percent: config.limits.warning_threshold_percent,
        },
        map_matching: MapMatchingSettingsInfo {
            provider: config.map_matching.provider.clone(),
            enabled: config.map_matching.enabled,
            timeout_ms: config.map_matching.timeout_ms,
            rate_limit_per_minute: config.map_matching.rate_limit_per_minute,
            circuit_breaker_failures: config.map_matching.circuit_breaker_failures,
            circuit_breaker_reset_secs: config.map_matching.circuit_breaker_reset_secs,
        },
        email: EmailSettingsInfo {
            enabled: config.email.enabled,
            provider: config.email.provider.clone(),
            sender_email: config.email.sender_email.clone(),
            sender_name: config.email.sender_name.clone(),
            template_style: config.email.template_style.clone(),
        },
        fcm: FcmSettingsInfo {
            enabled: config.fcm.enabled,
            project_id: config.fcm.project_id.clone(),
            timeout_ms: config.fcm.timeout_ms,
            max_retries: config.fcm.max_retries,
            high_priority: config.fcm.high_priority,
        },
        frontend: FrontendSettingsInfo {
            enabled: config.frontend.enabled,
            base_dir: config.frontend.base_dir.clone(),
            staging_hostname: config.frontend.staging_hostname.clone(),
            production_hostname: config.frontend.production_hostname.clone(),
            default_environment: config.frontend.default_environment.clone(),
            immutable_cache_max_age: config.frontend.immutable_cache_max_age,
            mutable_cache_max_age: config.frontend.mutable_cache_max_age,
        },
    };

    info!(
        user_id = %system_auth.user_id,
        "Retrieved system settings"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Get feature flags.
///
/// GET /api/admin/v1/system/feature-flags
///
/// Returns current feature flag settings.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn get_feature_flags(
    State(state): State<AppState>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can view feature flags
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let config = &state.config;

    let response = FeatureFlagsResponse {
        features: FeatureFlagsInfo {
            geofences_enabled: config.features.geofences_enabled,
            proximity_alerts_enabled: config.features.proximity_alerts_enabled,
            webhooks_enabled: config.features.webhooks_enabled,
            movement_tracking_enabled: config.features.movement_tracking_enabled,
            b2b_enabled: config.features.b2b_enabled,
            geofence_events_enabled: config.features.geofence_events_enabled,
        },
        auth: AuthTogglesInfo {
            registration_enabled: config.auth_toggles.registration_enabled,
            invite_only: config.auth_toggles.invite_only,
            oauth_only: config.auth_toggles.oauth_only,
            google_enabled: !config.oauth.google_client_id.is_empty(),
            apple_enabled: !config.oauth.apple_client_id.is_empty(),
        },
    };

    info!(
        user_id = %system_auth.user_id,
        "Retrieved feature flags"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Get rate limits.
///
/// GET /api/admin/v1/system/rate-limits
///
/// Returns current rate limit settings.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn get_rate_limits(
    State(state): State<AppState>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can view rate limits
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let config = &state.config;

    let response = RateLimitsResponse {
        rate_limit_per_minute: config.security.rate_limit_per_minute,
        export_rate_limit_per_hour: config.security.export_rate_limit_per_hour,
        forgot_password_rate_limit_per_hour: config.security.forgot_password_rate_limit_per_hour,
        request_verification_rate_limit_per_hour: config
            .security
            .request_verification_rate_limit_per_hour,
        map_matching_rate_limit_per_minute: config.map_matching.rate_limit_per_minute,
    };

    info!(
        user_id = %system_auth.user_id,
        "Retrieved rate limits"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Get maintenance mode status.
///
/// GET /api/admin/v1/system/maintenance
///
/// Returns current maintenance mode status.
/// Requires any system role.
#[axum::debug_handler(state = AppState)]
async fn get_maintenance_mode(
    _system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    let maintenance = MAINTENANCE_MODE
        .read()
        .map_err(|_| ApiError::Internal("Failed to read maintenance state".to_string()))?;

    let response = match maintenance.as_ref() {
        Some(state) => MaintenanceModeResponse {
            enabled: state.enabled,
            message: state.message.clone(),
            enabled_at: Some(state.enabled_at),
            estimated_end: state.estimated_end,
        },
        None => MaintenanceModeResponse {
            enabled: false,
            message: None,
            enabled_at: None,
            estimated_end: None,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Toggle maintenance mode.
///
/// POST /api/admin/v1/system/maintenance
///
/// Enables or disables maintenance mode.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn toggle_maintenance_mode(
    system_auth: SystemRoleAuth,
    Json(request): Json<ToggleMaintenanceModeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can toggle maintenance mode
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let mut maintenance = MAINTENANCE_MODE
        .write()
        .map_err(|_| ApiError::Internal("Failed to write maintenance state".to_string()))?;

    let now = chrono::Utc::now();

    if request.enabled {
        *maintenance = Some(MaintenanceState {
            enabled: true,
            message: request.message.clone(),
            enabled_at: now,
            estimated_end: request.estimated_end,
        });

        info!(
            user_id = %system_auth.user_id,
            message = ?request.message,
            estimated_end = ?request.estimated_end,
            "Maintenance mode enabled"
        );
    } else {
        *maintenance = None;

        info!(
            user_id = %system_auth.user_id,
            "Maintenance mode disabled"
        );
    }

    let response = MaintenanceModeResponse {
        enabled: request.enabled,
        message: request.message,
        enabled_at: if request.enabled { Some(now) } else { None },
        estimated_end: request.estimated_end,
    };

    Ok((StatusCode::OK, Json(response)))
}

// ============================================================================
// Feature Flag Mutation API (FR-9.5)
// ============================================================================

/// Update a feature flag.
///
/// PUT /api/admin/v1/system/feature-flags/:flag_id
///
/// Updates the enabled state of a feature flag.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn update_feature_flag(
    State(state): State<AppState>,
    Path(flag_id): Path<Uuid>,
    system_auth: SystemRoleAuth,
    Json(request): Json<UpdateFeatureFlagRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can update feature flags
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let repo = SystemConfigRepository::new(state.pool.clone());

    // Get the flag first to get its key
    let flag = repo
        .get_feature_flag_by_id(flag_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Feature flag not found".to_string()))?;

    // Update the flag
    let updated = repo
        .update_feature_flag(&flag.flag_key, request.enabled, system_auth.user_id)
        .await?;

    info!(
        user_id = %system_auth.user_id,
        flag_key = %updated.flag_key,
        enabled = updated.enabled,
        "Updated feature flag"
    );

    let response = FeatureFlagResponse {
        id: updated.id,
        flag_key: updated.flag_key,
        enabled: updated.enabled,
        description: updated.description,
        category: updated.category,
        updated_at: updated.updated_at,
    };

    Ok((StatusCode::OK, Json(response)))
}

// ============================================================================
// Rate Limits Mutation API (FR-9.4)
// ============================================================================

/// Update rate limits.
///
/// PUT /api/admin/v1/system/rate-limits
///
/// Updates rate limit configurations.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn update_rate_limits(
    State(state): State<AppState>,
    system_auth: SystemRoleAuth,
    Json(request): Json<UpdateRateLimitsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can update rate limits
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    // Validate request
    request.validate().map_err(|e| ApiError::Validation(e.to_string()))?;

    let repo = SystemConfigRepository::new(state.pool.clone());
    let mut updated_keys = Vec::new();

    // Update each rate limit if provided
    if let Some(limit) = request.rate_limit_per_minute {
        repo.update_rate_limit("api_general", limit as i32, system_auth.user_id)
            .await?;
        updated_keys.push("api_general");
    }

    if let Some(limit) = request.export_rate_limit_per_hour {
        repo.update_rate_limit("export", limit as i32, system_auth.user_id)
            .await?;
        updated_keys.push("export");
    }

    if let Some(limit) = request.forgot_password_rate_limit_per_hour {
        repo.update_rate_limit("forgot_password", limit as i32, system_auth.user_id)
            .await?;
        updated_keys.push("forgot_password");
    }

    if let Some(limit) = request.request_verification_rate_limit_per_hour {
        repo.update_rate_limit("request_verification", limit as i32, system_auth.user_id)
            .await?;
        updated_keys.push("request_verification");
    }

    if let Some(limit) = request.map_matching_rate_limit_per_minute {
        repo.update_rate_limit("map_matching", limit as i32, system_auth.user_id)
            .await?;
        updated_keys.push("map_matching");
    }

    info!(
        user_id = %system_auth.user_id,
        updated_keys = ?updated_keys,
        "Updated rate limits"
    );

    // Return updated rate limits
    let limits = repo.list_rate_limits().await?;
    let response: Vec<RateLimitConfigItem> = limits
        .into_iter()
        .map(|l| RateLimitConfigItem {
            id: l.id,
            limit_key: l.limit_key,
            requests_per_period: l.requests_per_period,
            period_seconds: l.period_seconds,
            description: l.description,
            updated_at: l.updated_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

// ============================================================================
// Notification Templates API (FR-9.3)
// ============================================================================

/// List notification templates.
///
/// GET /api/admin/v1/system/templates
///
/// Returns all notification templates.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn list_notification_templates(
    State(state): State<AppState>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can view templates
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let repo = SystemConfigRepository::new(state.pool.clone());
    let templates = repo.list_notification_templates().await?;

    let response = NotificationTemplatesResponse {
        total: templates.len() as i64,
        templates: templates
            .into_iter()
            .map(|t| NotificationTemplate {
                id: t.id,
                template_key: t.template_key,
                template_type: t.template_type,
                title_template: t.title_template,
                body_template: t.body_template,
                data_schema: t.data_schema,
                is_active: t.is_active,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect(),
    };

    info!(
        user_id = %system_auth.user_id,
        count = response.total,
        "Listed notification templates"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Update a notification template.
///
/// PUT /api/admin/v1/system/templates/:template_id
///
/// Updates a notification template.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn update_notification_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
    system_auth: SystemRoleAuth,
    Json(request): Json<UpdateNotificationTemplateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can update templates
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    // Validate request
    request.validate().map_err(|e| ApiError::Validation(e.to_string()))?;

    let repo = SystemConfigRepository::new(state.pool.clone());

    // Verify template exists
    repo.get_notification_template(template_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Notification template not found".to_string()))?;

    // Update the template
    let updated = repo
        .update_notification_template(
            template_id,
            request.title_template.as_deref(),
            request.body_template.as_deref(),
            request.data_schema,
            request.is_active,
            system_auth.user_id,
        )
        .await?;

    info!(
        user_id = %system_auth.user_id,
        template_id = %template_id,
        template_key = %updated.template_key,
        "Updated notification template"
    );

    let response = NotificationTemplate {
        id: updated.id,
        template_key: updated.template_key,
        template_type: updated.template_type,
        title_template: updated.title_template,
        body_template: updated.body_template,
        data_schema: updated.data_schema,
        is_active: updated.is_active,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    };

    Ok((StatusCode::OK, Json(response)))
}

// ============================================================================
// Email Templates API (FR-9.6)
// ============================================================================

/// List email templates.
///
/// GET /api/admin/v1/system/email-templates
///
/// Returns all email templates.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn list_email_templates(
    State(state): State<AppState>,
    system_auth: SystemRoleAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can view email templates
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    let repo = SystemConfigRepository::new(state.pool.clone());
    let templates = repo.list_email_templates().await?;

    let response = EmailTemplatesResponse {
        total: templates.len() as i64,
        templates: templates
            .into_iter()
            .map(|t| EmailTemplate {
                id: t.id,
                template_key: t.template_key,
                subject_template: t.subject_template,
                body_html_template: t.body_html_template,
                body_text_template: t.body_text_template,
                data_schema: t.data_schema,
                is_active: t.is_active,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect(),
    };

    info!(
        user_id = %system_auth.user_id,
        count = response.total,
        "Listed email templates"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Update an email template.
///
/// PUT /api/admin/v1/system/email-templates/:template_id
///
/// Updates an email template.
/// Requires super_admin role.
#[axum::debug_handler(state = AppState)]
async fn update_email_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
    system_auth: SystemRoleAuth,
    Json(request): Json<UpdateEmailTemplateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Only super_admin can update email templates
    if !system_auth.is_super_admin() {
        return Err(ApiError::Forbidden(
            "Super admin access required".to_string(),
        ));
    }

    // Validate request
    request.validate().map_err(|e| ApiError::Validation(e.to_string()))?;

    let repo = SystemConfigRepository::new(state.pool.clone());

    // Verify template exists
    repo.get_email_template(template_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Email template not found".to_string()))?;

    // Update the template
    let updated = repo
        .update_email_template(
            template_id,
            request.subject_template.as_deref(),
            request.body_html_template.as_deref(),
            request.body_text_template.as_deref(),
            request.data_schema,
            request.is_active,
            system_auth.user_id,
        )
        .await?;

    info!(
        user_id = %system_auth.user_id,
        template_id = %template_id,
        template_key = %updated.template_key,
        "Updated email template"
    );

    let response = EmailTemplate {
        id: updated.id,
        template_key: updated.template_key,
        subject_template: updated.subject_template,
        body_html_template: updated.body_html_template,
        body_text_template: updated.body_text_template,
        data_schema: updated.data_schema,
        is_active: updated.is_active,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }

    #[test]
    fn test_maintenance_mode_default() {
        // Reset maintenance mode for test
        if let Ok(mut m) = MAINTENANCE_MODE.write() {
            *m = None;
        }

        let maintenance = MAINTENANCE_MODE.read().unwrap();
        assert!(maintenance.is_none());
    }
}
