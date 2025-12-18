use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::config::Config;
use crate::middleware::{
    auth_rate_limit_middleware, metrics_handler, metrics_middleware, rate_limit_middleware,
    require_admin, require_auth, require_b2b, require_geofence_events, require_geofences,
    require_movement_tracking, require_proximity_alerts, require_webhooks,
    security_headers_middleware, trace_id, version_check, AuthRateLimiterState,
    ExportRateLimiterState, RateLimiterState,
};
use crate::routes::{
    admin, admin_geofences, admin_groups, admin_locations, admin_managed_users,
    admin_unlock_requests, admin_users, analytics, api_keys, app_usage, audit_logs, auth,
    bulk_import, compliance, dashboard, data_subject_requests, device_policies, device_settings,
    devices, enrollment, enrollment_tokens, fleet, frontend, geofence_events, geofences, groups,
    health, invites, locations, movement_events, openapi, org_invitations, org_webhooks,
    organization_settings, organizations, permissions, privacy, proximity_alerts, public_config,
    roles, system_config, system_roles, trips, users, versioning, webhooks,
};
use crate::services::cookies::CookieHelper;
use crate::services::fcm::FcmNotificationService;
use crate::services::map_matching::MapMatchingClient;
use domain::services::{MockNotificationService, NotificationService};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub rate_limiter: Option<Arc<RateLimiterState>>,
    /// Export rate limiter for per-organization audit log exports
    pub export_rate_limiter: Option<Arc<ExportRateLimiterState>>,
    /// Forgot password rate limiter (per-IP)
    pub forgot_password_rate_limiter: Option<Arc<AuthRateLimiterState>>,
    /// Request verification rate limiter (per-IP)
    pub request_verification_rate_limiter: Option<Arc<AuthRateLimiterState>>,
    /// Shared map-matching client (None if disabled or failed to initialize)
    pub map_matching_client: Option<Arc<MapMatchingClient>>,
    /// Notification service for push notifications
    pub notification_service: Arc<dyn NotificationService>,
    /// Cookie helper for httpOnly authentication
    pub cookie_helper: Arc<CookieHelper>,
}

pub fn create_app(config: Config, pool: PgPool) -> Router {
    let config = Arc::new(config);

    // Create rate limiter if rate limiting is enabled (rate_limit_per_minute > 0)
    let rate_limiter = if config.security.rate_limit_per_minute > 0 {
        Some(Arc::new(RateLimiterState::new(
            config.security.rate_limit_per_minute,
        )))
    } else {
        None
    };

    // Create export rate limiter for per-organization export limiting
    let export_rate_limiter = if config.security.export_rate_limit_per_hour > 0 {
        Some(Arc::new(ExportRateLimiterState::new(
            config.security.export_rate_limit_per_hour,
        )))
    } else {
        None
    };

    // Create auth rate limiters for forgot-password and request-verification endpoints
    let forgot_password_rate_limiter = if config.security.forgot_password_rate_limit_per_hour > 0 {
        Some(Arc::new(AuthRateLimiterState::new(
            config.security.forgot_password_rate_limit_per_hour,
            "forgot-password",
        )))
    } else {
        None
    };

    let request_verification_rate_limiter =
        if config.security.request_verification_rate_limit_per_hour > 0 {
            Some(Arc::new(AuthRateLimiterState::new(
                config.security.request_verification_rate_limit_per_hour,
                "request-verification",
            )))
        } else {
            None
        };

    // Create map-matching client if enabled and configured
    let map_matching_client = if config.map_matching.enabled && !config.map_matching.url.is_empty()
    {
        match MapMatchingClient::new(config.map_matching.clone()) {
            Ok(client) => Some(Arc::new(client)),
            Err(e) => {
                tracing::error!(error = %e, "Failed to create map-matching client");
                None
            }
        }
    } else {
        tracing::debug!("Map-matching is disabled or not configured");
        None
    };

    // Create notification service (FCM if enabled and configured, otherwise mock)
    let notification_service: Arc<dyn NotificationService> = if config.fcm.enabled {
        match FcmNotificationService::new(config.fcm.clone()) {
            Ok(service) => {
                tracing::info!(
                    project_id = %config.fcm.project_id,
                    high_priority = %config.fcm.high_priority,
                    "FCM notification service initialized"
                );
                Arc::new(service)
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create FCM notification service, falling back to mock");
                Arc::new(MockNotificationService::new())
            }
        }
    } else {
        tracing::info!("Notification service initialized (mock mode - FCM disabled)");
        Arc::new(MockNotificationService::new())
    };

    // Create cookie helper for httpOnly authentication
    let cookie_helper = Arc::new(CookieHelper::new(
        config.cookies.clone(),
        config.jwt.access_token_expiry_secs,
        config.jwt.refresh_token_expiry_secs,
    ));
    if config.cookies.enabled {
        tracing::info!(
            secure = %config.cookies.secure,
            same_site = %config.cookies.same_site,
            "Cookie authentication enabled"
        );
    }

    let state = AppState {
        pool,
        config: config.clone(),
        rate_limiter,
        export_rate_limiter,
        forgot_password_rate_limiter,
        request_verification_rate_limiter,
        map_matching_client,
        notification_service,
        cookie_helper,
    };

    // Build CORS layer based on configuration
    // When cookie authentication is enabled, we need credentials support,
    // which requires specific origins (not wildcard "*")
    let cors = if config.cookies.enabled {
        // Cookie auth enabled: must use specific origins with credentials
        use tower_http::cors::AllowOrigin;

        if config.security.cors_origins.is_empty()
            || config.security.cors_origins.iter().any(|o| o == "*")
        {
            tracing::warn!(
                "Cookie authentication is enabled but CORS origins are not configured. \
                 Set PM__SECURITY__CORS_ORIGINS to specific origins for credentials to work."
            );
            // Fall back to permissive (credentials won't work properly)
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            // Specific origins with credentials support
            let origins: Vec<_> = config
                .security
                .cors_origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_credentials(true)
        }
    } else if config.security.cors_origins.is_empty()
        || config.security.cors_origins.iter().any(|o| o == "*")
    {
        // Default: allow any origin (for development or when "*" is specified)
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        // Production: only allow specified origins (no credentials)
        use tower_http::cors::AllowOrigin;
        let origins: Vec<_> = config
            .security
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Core protected routes (always enabled - require API key authentication)
    let core_protected_routes = Router::new()
        // Device routes (v1)
        .route("/api/v1/devices/register", post(devices::register_device))
        .route("/api/v1/devices", get(devices::get_group_devices))
        .route("/api/v1/devices/:device_id", delete(devices::delete_device))
        // Location routes (v1)
        .route("/api/v1/locations", post(locations::upload_location))
        .route("/api/v1/locations/batch", post(locations::upload_batch))
        // Device location history (v1)
        .route(
            "/api/v1/devices/:device_id/locations",
            get(locations::get_location_history),
        )
        // Privacy routes (v1) - GDPR compliance
        .route(
            "/api/v1/devices/:device_id/data-export",
            get(privacy::export_device_data),
        )
        .route(
            "/api/v1/devices/:device_id/data",
            delete(privacy::delete_device_data),
        );

    // Movement tracking routes (feature toggle: movement_tracking_enabled)
    let movement_routes = Router::new()
        // Movement event routes (v1)
        .route(
            "/api/v1/movement-events",
            post(movement_events::create_movement_event),
        )
        .route(
            "/api/v1/movement-events/batch",
            post(movement_events::create_movement_events_batch),
        )
        .route(
            "/api/v1/devices/:device_id/movement-events",
            get(movement_events::get_device_movement_events),
        )
        // Trip routes (v1)
        .route("/api/v1/trips", post(trips::create_trip))
        .route("/api/v1/trips/:trip_id", patch(trips::update_trip_state))
        .route(
            "/api/v1/trips/:trip_id/movement-events",
            get(trips::get_trip_movement_events),
        )
        .route("/api/v1/trips/:trip_id/path", get(trips::get_trip_path))
        .route(
            "/api/v1/trips/:trip_id/correct-path",
            post(trips::trigger_path_correction),
        )
        .route(
            "/api/v1/devices/:device_id/trips",
            get(trips::get_device_trips),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_movement_tracking,
        ));

    // Geofence routes (feature toggle: geofences_enabled)
    let geofence_routes = Router::new()
        .route("/api/v1/geofences", post(geofences::create_geofence))
        .route("/api/v1/geofences", get(geofences::list_geofences))
        .route(
            "/api/v1/geofences/:geofence_id",
            get(geofences::get_geofence),
        )
        .route(
            "/api/v1/geofences/:geofence_id",
            patch(geofences::update_geofence),
        )
        .route(
            "/api/v1/geofences/:geofence_id",
            delete(geofences::delete_geofence),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_geofences,
        ));

    // Proximity alert routes (feature toggle: proximity_alerts_enabled)
    let proximity_routes = Router::new()
        .route(
            "/api/v1/proximity-alerts",
            post(proximity_alerts::create_proximity_alert),
        )
        .route(
            "/api/v1/proximity-alerts",
            get(proximity_alerts::list_proximity_alerts),
        )
        .route(
            "/api/v1/proximity-alerts/:alert_id",
            get(proximity_alerts::get_proximity_alert),
        )
        .route(
            "/api/v1/proximity-alerts/:alert_id",
            patch(proximity_alerts::update_proximity_alert),
        )
        .route(
            "/api/v1/proximity-alerts/:alert_id",
            delete(proximity_alerts::delete_proximity_alert),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_proximity_alerts,
        ));

    // Webhook routes (feature toggle: webhooks_enabled)
    let webhook_routes = Router::new()
        .route("/api/v1/webhooks", post(webhooks::create_webhook))
        .route("/api/v1/webhooks", get(webhooks::list_webhooks))
        .route("/api/v1/webhooks/:webhook_id", get(webhooks::get_webhook))
        .route(
            "/api/v1/webhooks/:webhook_id",
            put(webhooks::update_webhook),
        )
        .route(
            "/api/v1/webhooks/:webhook_id",
            delete(webhooks::delete_webhook),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_webhooks,
        ));

    // Geofence event routes (feature toggle: geofence_events_enabled)
    let geofence_event_routes = Router::new()
        .route(
            "/api/v1/geofence-events",
            post(geofence_events::create_geofence_event),
        )
        .route(
            "/api/v1/geofence-events",
            get(geofence_events::list_geofence_events),
        )
        .route(
            "/api/v1/geofence-events/:event_id",
            get(geofence_events::get_geofence_event),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_geofence_events,
        ));

    // Protected routes (require API key authentication)
    // Merge all feature routes with core routes
    // Middleware order: feature check -> auth -> rate limiting
    let protected_routes = Router::new()
        .merge(core_protected_routes)
        .merge(movement_routes)
        .merge(geofence_routes)
        .merge(proximity_routes)
        .merge(webhook_routes)
        .merge(geofence_event_routes)
        // Rate limiting runs after auth (needs API key ID from auth)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        // Auth runs first (outermost layer = runs first)
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Core admin routes (always enabled - require admin API key)
    // Note: admin_managed_users is here (not in b2b_admin_routes) because it needs
    // to be accessible by both org admins AND non-org admins. Non-org admins
    // manage users not in any organization, while org admins manage their org's users.
    let core_admin_routes = Router::new()
        .route(
            "/api/v1/admin/devices/inactive",
            delete(admin::delete_inactive_devices),
        )
        .route(
            "/api/v1/admin/devices/:device_id/reactivate",
            post(admin::reactivate_device),
        )
        .route("/api/v1/admin/stats", get(admin::get_admin_stats))
        // Admin managed users routes (Epic 9 - user location, geofences, tracking)
        // Accessible by both org admins and non-org admins
        .nest("/api/admin/v1/users", admin_managed_users::router());

    // B2B/Organization admin routes (feature toggle: b2b_enabled)
    let b2b_admin_routes = Router::new()
        // Organization management routes (Story 13.1)
        .route(
            "/api/admin/v1/organizations",
            post(organizations::create_organization).get(organizations::list_organizations),
        )
        .route(
            "/api/admin/v1/organizations/:org_id",
            get(organizations::get_organization)
                .put(organizations::update_organization)
                .delete(organizations::delete_organization),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/usage",
            get(organizations::get_organization_usage),
        )
        // Organization suspend/reactivate routes (Story AP-2.7)
        .route(
            "/api/admin/v1/organizations/:org_id/suspend",
            post(organizations::suspend_organization),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/reactivate",
            post(organizations::reactivate_organization),
        )
        // Organization user management routes (Story 13.2)
        .route(
            "/api/admin/v1/organizations/:org_id/users",
            post(organizations::add_org_user).get(organizations::list_org_users),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/users/:user_id",
            put(organizations::update_org_user).delete(organizations::remove_org_user),
        )
        // Device policy management routes (Story 13.3)
        .route(
            "/api/admin/v1/organizations/:org_id/policies",
            post(device_policies::create_policy).get(device_policies::list_policies),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/policies/:policy_id",
            get(device_policies::get_policy)
                .put(device_policies::update_policy)
                .delete(device_policies::delete_policy),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/policies/:policy_id/apply",
            post(device_policies::apply_policy),
        )
        // Policy unapply endpoint (Story 14.5)
        .route(
            "/api/admin/v1/organizations/:org_id/policies/:policy_id/unapply",
            post(device_policies::unapply_policy),
        )
        // Enrollment token management routes (Story 13.4)
        .route(
            "/api/admin/v1/organizations/:org_id/enrollment-tokens",
            post(enrollment_tokens::create_enrollment_token)
                .get(enrollment_tokens::list_enrollment_tokens),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id",
            get(enrollment_tokens::get_enrollment_token)
                .delete(enrollment_tokens::revoke_enrollment_token),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id/qr",
            get(enrollment_tokens::get_enrollment_token_qr),
        )
        // Fleet management routes (Story 13.7)
        .nest(
            "/api/admin/v1/organizations/:org_id/devices",
            fleet::router(),
        )
        // Bulk import routes (Story 13.8)
        .nest(
            "/api/admin/v1/organizations/:org_id/devices/bulk",
            bulk_import::router(),
        )
        // Audit log routes (Story 13.9, 13.10)
        .nest(
            "/api/admin/v1/organizations/:org_id/audit-logs",
            audit_logs::router(),
        )
        // Data subject request routes (AP-11.4-6)
        .nest(
            "/api/admin/v1/organizations/:org_id/data-requests",
            data_subject_requests::router(),
        )
        // Compliance routes (AP-11.7-8)
        .nest(
            "/api/admin/v1/organizations/:org_id/compliance",
            compliance::router(),
        )
        // Dashboard metrics (Story 14.1)
        .route(
            "/api/admin/v1/organizations/:org_id/dashboard",
            get(dashboard::get_dashboard_metrics),
        )
        // Admin user management routes (Story 14.3)
        .nest(
            "/api/admin/v1/organizations/:org_id/admin-users",
            admin_users::router(),
        )
        // Admin group management routes (Story 14.4)
        .nest(
            "/api/admin/v1/organizations/:org_id/groups",
            admin_groups::router(),
        )
        // Organization settings routes
        .route(
            "/api/admin/v1/organizations/:org_id/settings",
            get(organization_settings::get_organization_settings)
                .put(organization_settings::update_organization_settings),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/settings/verify-pin",
            post(organization_settings::verify_unlock_pin),
        )
        // Organization API keys routes
        .route(
            "/api/admin/v1/organizations/:org_id/api-keys",
            post(api_keys::create_api_key).get(api_keys::list_api_keys),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/api-keys/:key_id",
            get(api_keys::get_api_key)
                .patch(api_keys::update_api_key)
                .delete(api_keys::revoke_api_key),
        )
        // Organization member invitations routes
        .route(
            "/api/admin/v1/organizations/:org_id/invitations",
            post(org_invitations::create_invitation).get(org_invitations::list_invitations),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/invitations/:invite_id",
            get(org_invitations::get_invitation).delete(org_invitations::revoke_invitation),
        )
        // Organization webhooks routes
        .route(
            "/api/admin/v1/organizations/:org_id/webhooks",
            post(org_webhooks::create_webhook).get(org_webhooks::list_webhooks),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/webhooks/:webhook_id",
            get(org_webhooks::get_webhook)
                .put(org_webhooks::update_webhook)
                .delete(org_webhooks::delete_webhook),
        )
        // Organization webhook test, deliveries, and stats routes (AP-7.5-7.8)
        .route(
            "/api/admin/v1/organizations/:org_id/webhooks/:webhook_id/test",
            post(org_webhooks::test_webhook),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/webhooks/:webhook_id/deliveries",
            get(org_webhooks::list_deliveries),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/webhooks/:webhook_id/deliveries/:delivery_id/retry",
            post(org_webhooks::retry_delivery),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/webhooks/:webhook_id/stats",
            get(org_webhooks::get_webhook_stats),
        )
        // Organization permissions routes (Story AP-1.1)
        .nest(
            "/api/admin/v1/organizations/:org_id/permissions",
            permissions::router(),
        )
        // Organization role management routes (Story AP-1.2, AP-1.3)
        .nest("/api/admin/v1/organizations/:org_id/roles", roles::router())
        // Admin geofence management routes (Story AP-6)
        .nest(
            "/api/admin/v1/organizations/:org_id/geofences",
            admin_geofences::router(),
        )
        // Admin location management routes (Story AP-6)
        .nest(
            "/api/admin/v1/organizations/:org_id/locations",
            admin_locations::router(),
        )
        // Admin device location routes (Story AP-6)
        .nest(
            "/api/admin/v1/organizations/:org_id/devices",
            admin_locations::device_location_router(),
        )
        // Admin geofence events routes (Story AP-6)
        .nest(
            "/api/admin/v1/organizations/:org_id/geofence-events",
            admin_geofences::geofence_events_router(),
        )
        // Admin location analytics routes (Story AP-6)
        .nest(
            "/api/admin/v1/organizations/:org_id/location-analytics",
            admin_geofences::location_analytics_router(),
        )
        // Admin unlock request management routes (Story AP-8)
        .nest(
            "/api/admin/v1/organizations/:org_id/unlock-requests",
            admin_unlock_requests::router(),
        )
        // App usage routes - device level (Story AP-8.1, AP-8.2)
        .nest(
            "/api/admin/v1/organizations/:org_id/devices/:device_id/app-usage",
            app_usage::device_router(),
        )
        // App usage routes - organization level analytics (Story AP-8.7)
        .nest(
            "/api/admin/v1/organizations/:org_id/app-usage",
            app_usage::org_router(),
        )
        // Analytics routes (Story AP-10.1, AP-10.2, AP-10.3)
        .nest(
            "/api/admin/v1/organizations/:org_id/analytics",
            analytics::router(),
        )
        // Report generation routes (Story AP-10.4, AP-10.5, AP-10.6, AP-10.7)
        .nest(
            "/api/admin/v1/organizations/:org_id/reports",
            analytics::reports_router(),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), require_b2b));

    // Admin routes (require admin API key)
    let admin_routes = Router::new()
        .merge(core_admin_routes)
        .merge(b2b_admin_routes)
        // Rate limiting for admin routes (separate, higher limit could be configured)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        // Admin auth runs first
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin));

    // System role management routes (require JWT auth with system roles)
    // These routes use the SystemRoleAuth extractor which validates JWT and checks system roles
    let system_role_routes =
        Router::new().nest("/api/admin/v1/system-roles", system_roles::router());

    // System configuration routes (require JWT auth with super_admin role)
    // AP-9: System Configuration endpoints
    let system_config_routes = Router::new().nest("/api/admin/v1/system", system_config::router());

    // Legacy routes - redirect to v1 with 301 Moved Permanently
    // These don't require auth since they just redirect
    let legacy_routes = Router::new()
        .route(
            "/api/devices/register",
            post(versioning::redirect_devices_register),
        )
        .route("/api/devices", get(versioning::redirect_devices_list))
        .route("/api/locations", post(versioning::redirect_locations))
        .route(
            "/api/locations/batch",
            post(versioning::redirect_locations_batch),
        );

    // Auth routes (public, no authentication required)
    // Non-rate-limited auth routes
    let auth_routes_base = Router::new()
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/oauth", post(auth::oauth_login))
        .route("/api/v1/auth/refresh", post(auth::refresh))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/reset-password", post(auth::reset_password))
        .route("/api/v1/auth/verify-email", post(auth::verify_email));

    // Rate-limited forgot-password route (5/hour per IP)
    let forgot_password_routes = if let Some(ref limiter) = state.forgot_password_rate_limiter {
        Router::new()
            .route("/api/v1/auth/forgot-password", post(auth::forgot_password))
            .route_layer(middleware::from_fn_with_state(
                limiter.clone(),
                auth_rate_limit_middleware,
            ))
    } else {
        Router::new().route("/api/v1/auth/forgot-password", post(auth::forgot_password))
    };

    // Rate-limited request-verification route (3/hour per IP)
    // Note: This route also requires JWT auth (user must be logged in)
    let request_verification_routes =
        if let Some(ref limiter) = state.request_verification_rate_limiter {
            Router::new()
                .route(
                    "/api/v1/auth/request-verification",
                    post(auth::request_verification),
                )
                .route_layer(middleware::from_fn_with_state(
                    limiter.clone(),
                    auth_rate_limit_middleware,
                ))
        } else {
            Router::new().route(
                "/api/v1/auth/request-verification",
                post(auth::request_verification),
            )
        };

    // Combine all auth routes
    let auth_routes = auth_routes_base
        .merge(forgot_password_routes)
        .merge(request_verification_routes);

    // User profile routes (require JWT authentication)
    // The UserAuth extractor handles JWT validation directly
    let user_routes = Router::new()
        .route("/api/v1/users/me", get(users::get_current_user))
        .route("/api/v1/users/me", put(users::update_current_user))
        // Registration group status endpoint (UGM-1.2)
        .route(
            "/api/v1/devices/me/registration-group",
            get(devices::get_registration_group_status),
        )
        // User's linked devices endpoint (UGM-1.3)
        .route("/api/v1/devices/me", get(devices::get_my_devices))
        // Device binding endpoints
        .route(
            "/api/v1/users/:user_id/devices/:device_id/link",
            post(users::link_device),
        )
        .route(
            "/api/v1/users/:user_id/devices",
            get(users::list_user_devices),
        )
        .route(
            "/api/v1/users/:user_id/devices/:device_id/unlink",
            delete(users::unlink_device),
        )
        .route(
            "/api/v1/users/:user_id/devices/:device_id/transfer",
            post(users::transfer_device),
        )
        // Device settings endpoints (Story 12.2, 12.3, 12.4, 12.5)
        .route(
            "/api/v1/devices/:device_id/settings",
            get(device_settings::get_device_settings).put(device_settings::update_device_settings),
        )
        .route(
            "/api/v1/devices/:device_id/settings/locks",
            get(device_settings::get_setting_locks).put(device_settings::bulk_update_locks),
        )
        .route(
            "/api/v1/devices/:device_id/settings/:key",
            put(device_settings::update_device_setting),
        )
        .route(
            "/api/v1/devices/:device_id/settings/:key/lock",
            post(device_settings::lock_setting).delete(device_settings::unlock_setting),
        )
        // Settings sync endpoint (Story 12.7)
        .route(
            "/api/v1/devices/:device_id/settings/sync",
            post(device_settings::sync_settings),
        )
        // Settings history endpoint
        .route(
            "/api/v1/devices/:device_id/settings/history",
            get(device_settings::get_settings_history),
        )
        // Unlock request endpoint (Story 12.6)
        .route(
            "/api/v1/devices/:device_id/settings/:key/unlock-request",
            post(device_settings::create_unlock_request),
        )
        // Respond to unlock request (Story 12.6)
        .route(
            "/api/v1/unlock-requests/:request_id",
            put(device_settings::respond_to_unlock_request),
        );

    // Group management routes (require JWT authentication)
    // The UserAuth extractor handles JWT validation directly
    let group_routes = Router::new()
        .route("/api/v1/groups", post(groups::create_group))
        .route("/api/v1/groups", get(groups::list_groups))
        .route("/api/v1/groups/:group_id", get(groups::get_group))
        .route("/api/v1/groups/:group_id", put(groups::update_group))
        .route("/api/v1/groups/:group_id", delete(groups::delete_group))
        // Membership management (Story 11.2)
        .route(
            "/api/v1/groups/:group_id/members",
            get(groups::list_members),
        )
        .route(
            "/api/v1/groups/:group_id/members/:user_id",
            get(groups::get_member),
        )
        .route(
            "/api/v1/groups/:group_id/members/:user_id",
            delete(groups::remove_member),
        )
        // Role management (Story 11.3)
        .route(
            "/api/v1/groups/:group_id/members/:user_id/role",
            put(groups::update_member_role),
        )
        // Invite management (Story 11.4)
        .route(
            "/api/v1/groups/:group_id/invites",
            post(invites::create_invite),
        )
        .route(
            "/api/v1/groups/:group_id/invites",
            get(invites::list_invites),
        )
        .route(
            "/api/v1/groups/:group_id/invites/:invite_id",
            delete(invites::revoke_invite),
        )
        // Join group with invite code (Story 11.5)
        .route("/api/v1/groups/join", post(groups::join_group))
        // Migrate registration group to authenticated group (Story UGM-2.2)
        .route(
            "/api/v1/groups/migrate",
            post(groups::migrate_registration_group),
        )
        // Group devices (Story 12.7) - JWT-authenticated endpoint for member devices
        .route(
            "/api/v1/groups/:group_id/devices",
            get(groups::get_group_devices),
        )
        // Ownership transfer (Story 11.6)
        .route(
            "/api/v1/groups/:group_id/transfer",
            post(groups::transfer_ownership),
        )
        // Unlock requests for group (Story 12.6)
        .route(
            "/api/v1/groups/:group_id/unlock-requests",
            get(device_settings::list_unlock_requests),
        );

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/health", get(health::health_check))
        // Public config - feature flags and auth configuration
        .route(
            "/api/v1/config/public",
            get(public_config::get_public_config),
        )
        // Public invite info (Story 11.4)
        .route("/api/v1/invites/:code", get(invites::get_invite_info))
        // Alias for Android app compatibility
        .route(
            "/api/v1/invites/:code/validate",
            get(invites::get_invite_info),
        )
        .route("/api/health/ready", get(health::ready))
        .route("/api/health/live", get(health::live))
        .route("/metrics", get(metrics_handler));

    // B2B public routes (no auth but requires B2B feature enabled)
    let b2b_public_routes = Router::new()
        // Device enrollment (Story 13.5) - token is the auth, requires B2B
        .route("/api/v1/devices/enroll", post(enrollment::enroll_device))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_b2b,
        ));

    // OpenAPI documentation routes (public, no auth)
    let openapi_routes = Router::new()
        .route("/api/docs", get(openapi::swagger_ui_redirect))
        .route("/api/docs/", get(openapi::swagger_ui))
        .route("/api/docs/*path", get(openapi::swagger_ui))
        .route("/api/docs/openapi.yaml", get(openapi::openapi_spec));

    // Merge all routes
    let mut app = Router::new()
        .merge(public_routes)
        .merge(b2b_public_routes)
        .merge(auth_routes)
        .merge(user_routes)
        .merge(group_routes)
        .merge(openapi_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .merge(system_role_routes)
        .merge(system_config_routes)
        .merge(legacy_routes);

    // Add frontend serving as fallback if enabled
    // This must be added after API routes so they take precedence
    if config.frontend.enabled {
        tracing::info!(
            base_dir = %config.frontend.base_dir,
            staging_hostname = %config.frontend.staging_hostname,
            production_hostname = %config.frontend.production_hostname,
            default_environment = %config.frontend.default_environment,
            "Frontend static file serving enabled"
        );
        app = app.fallback(frontend::serve_frontend);
    }

    // Global middleware (order matters: bottom layers run first)
    app.layer(middleware::from_fn(security_headers_middleware)) // Security headers
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.server.request_timeout_secs,
        )))
        .layer(middleware::from_fn(metrics_middleware)) // Prometheus metrics
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(version_check)) // Client version compatibility check
        .layer(middleware::from_fn(trace_id)) // Request ID and logging
        .layer(cors)
        .with_state(state)
}
