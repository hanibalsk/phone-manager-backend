use axum::{
    middleware,
    routing::{delete, get, post},
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
use crate::middleware::{require_auth, trace_id};
use crate::routes::{devices, health, locations};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

pub fn create_app(config: Config, pool: PgPool) -> Router {
    let config = Arc::new(config);

    let state = AppState {
        pool,
        config: config.clone(),
    };

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Protected routes (require API key authentication)
    // Using /api/v1 prefix for versioned API
    let protected_routes = Router::new()
        // Device routes
        .route("/api/v1/devices/register", post(devices::register_device))
        .route("/api/v1/devices", get(devices::get_group_devices))
        .route("/api/v1/devices/:device_id", delete(devices::delete_device))
        // Location routes
        .route("/api/v1/locations", post(locations::upload_location))
        .route("/api/v1/locations/batch", post(locations::upload_batch))
        // Legacy routes (redirect to v1 in future, for now just alias)
        .route("/api/devices/register", post(devices::register_device))
        .route("/api/devices", get(devices::get_group_devices))
        .route("/api/locations", post(locations::upload_location))
        .route("/api/locations/batch", post(locations::upload_batch))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/health/ready", get(health::ready))
        .route("/api/health/live", get(health::live));

    // Merge all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // Global middleware (order matters: bottom layers run first)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.server.request_timeout_secs,
        )))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(trace_id)) // Request ID and logging
        .layer(cors)
        .with_state(state)
}
