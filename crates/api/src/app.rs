use axum::{
    middleware,
    routing::{delete, get, patch, post},
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
use crate::routes::{admin, devices, geofences, health, locations, movement_events, openapi, privacy, proximity_alerts, versioning};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub rate_limiter: Option<Arc<RateLimiterState>>,
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

    let state = AppState {
        pool,
        config: config.clone(),
        rate_limiter,
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

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/health", get(health::health_check))
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
