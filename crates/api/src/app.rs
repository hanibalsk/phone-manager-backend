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
    metrics_handler, metrics_middleware, rate_limit_middleware, require_admin, require_auth,
    security_headers_middleware, trace_id, RateLimiterState,
};
use crate::routes::{
    admin, auth, device_policies, device_settings, devices, enrollment, enrollment_tokens,
    geofences, groups, health, invites, locations, movement_events, openapi, organizations,
    privacy, proximity_alerts, trips, users, versioning,
};
use crate::services::map_matching::MapMatchingClient;
use domain::services::{MockNotificationService, NotificationService};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub rate_limiter: Option<Arc<RateLimiterState>>,
    /// Shared map-matching client (None if disabled or failed to initialize)
    pub map_matching_client: Option<Arc<MapMatchingClient>>,
    /// Notification service for push notifications
    pub notification_service: Arc<dyn NotificationService>,
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

    // Create notification service (using mock for now, can be replaced with FCM client later)
    let notification_service: Arc<dyn NotificationService> =
        Arc::new(MockNotificationService::new());
    tracing::info!("Notification service initialized (mock mode)");

    let state = AppState {
        pool,
        config: config.clone(),
        rate_limiter,
        map_matching_client,
        notification_service,
    };

    // Build CORS layer based on configuration
    let cors = if config.security.cors_origins.is_empty() {
        // Default: allow any origin (for development)
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        // Production: only allow specified origins
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

    // Protected routes (require API key authentication)
    // Using /api/v1 prefix for versioned API
    // Middleware order: auth runs first, then rate limiting (which needs the auth info)
    let protected_routes = Router::new()
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
        // Geofence routes (v1)
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
        // Proximity alert routes (v1)
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
        // Privacy routes (v1) - GDPR compliance
        .route(
            "/api/v1/devices/:device_id/data-export",
            get(privacy::export_device_data),
        )
        .route(
            "/api/v1/devices/:device_id/data",
            delete(privacy::delete_device_data),
        )
        // Rate limiting runs after auth (needs API key ID from auth)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        // Auth runs first (outermost layer = runs first)
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Admin routes (require admin API key)
    let admin_routes = Router::new()
        .route(
            "/api/v1/admin/devices/inactive",
            delete(admin::delete_inactive_devices),
        )
        .route(
            "/api/v1/admin/devices/:device_id/reactivate",
            post(admin::reactivate_device),
        )
        .route("/api/v1/admin/stats", get(admin::get_admin_stats))
        // Organization management routes (Story 13.1)
        .route(
            "/api/admin/v1/organizations",
            post(organizations::create_organization)
                .get(organizations::list_organizations),
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
        // Organization user management routes (Story 13.2)
        .route(
            "/api/admin/v1/organizations/:org_id/users",
            post(organizations::add_org_user)
                .get(organizations::list_org_users),
        )
        .route(
            "/api/admin/v1/organizations/:org_id/users/:user_id",
            put(organizations::update_org_user)
                .delete(organizations::remove_org_user),
        )
        // Device policy management routes (Story 13.3)
        .route(
            "/api/admin/v1/organizations/:org_id/policies",
            post(device_policies::create_policy)
                .get(device_policies::list_policies),
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
        // Rate limiting for admin routes (separate, higher limit could be configured)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        // Admin auth runs first
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin));

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
    let auth_routes = Router::new()
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/oauth", post(auth::oauth_login))
        .route("/api/v1/auth/refresh", post(auth::refresh))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route(
            "/api/v1/auth/forgot-password",
            post(auth::forgot_password),
        )
        .route(
            "/api/v1/auth/reset-password",
            post(auth::reset_password),
        )
        .route("/api/v1/auth/verify-email", post(auth::verify_email))
        // Request verification requires JWT auth (user must be logged in)
        .route(
            "/api/v1/auth/request-verification",
            post(auth::request_verification),
        );

    // User profile routes (require JWT authentication)
    // The UserAuth extractor handles JWT validation directly
    let user_routes = Router::new()
        .route("/api/v1/users/me", get(users::get_current_user))
        .route("/api/v1/users/me", put(users::update_current_user))
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
            get(device_settings::get_device_settings)
                .put(device_settings::update_device_settings),
        )
        .route(
            "/api/v1/devices/:device_id/settings/locks",
            get(device_settings::get_setting_locks)
                .put(device_settings::bulk_update_locks),
        )
        .route(
            "/api/v1/devices/:device_id/settings/:key",
            put(device_settings::update_device_setting),
        )
        .route(
            "/api/v1/devices/:device_id/settings/:key/lock",
            post(device_settings::lock_setting)
                .delete(device_settings::unlock_setting),
        )
        // Settings sync endpoint (Story 12.7)
        .route(
            "/api/v1/devices/:device_id/settings/sync",
            post(device_settings::sync_settings),
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
        // Public invite info (Story 11.4)
        .route("/api/v1/invites/:code", get(invites::get_invite_info))
        // Device enrollment (Story 13.5) - token is the auth
        .route("/api/v1/devices/enroll", post(enrollment::enroll_device))
        .route("/api/health/ready", get(health::ready))
        .route("/api/health/live", get(health::live))
        .route("/metrics", get(metrics_handler));

    // OpenAPI documentation routes (public, no auth)
    let openapi_routes = Router::new()
        .route("/api/docs", get(openapi::swagger_ui_redirect))
        .route("/api/docs/", get(openapi::swagger_ui))
        .route("/api/docs/*path", get(openapi::swagger_ui))
        .route("/api/docs/openapi.yaml", get(openapi::openapi_spec));

    // Merge all routes
    Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(user_routes)
        .merge(group_routes)
        .merge(openapi_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .merge(legacy_routes)
        // Global middleware (order matters: bottom layers run first)
        .layer(middleware::from_fn(security_headers_middleware)) // Security headers
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.server.request_timeout_secs,
        )))
        .layer(middleware::from_fn(metrics_middleware)) // Prometheus metrics
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(trace_id)) // Request ID and logging
        .layer(cors)
        .with_state(state)
}
