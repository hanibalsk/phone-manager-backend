use axum::{
    routing::{get, post},
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
use crate::routes::{devices, health, locations};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    #[allow(dead_code)] // Used in future stories for rate limiting, etc.
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

    // Build router
    Router::new()
        // Health routes (no auth required)
        .route("/api/health", get(health::health_check))
        .route("/api/health/ready", get(health::ready))
        .route("/api/health/live", get(health::live))
        // Device routes
        .route("/api/devices/register", post(devices::register_device))
        .route("/api/devices", get(devices::get_group_devices))
        // Location routes
        .route("/api/locations", post(locations::upload_location))
        .route("/api/locations/batch", post(locations::upload_batch))
        // Middleware
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.server.request_timeout_secs,
        )))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
