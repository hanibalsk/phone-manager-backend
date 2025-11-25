//! Health check endpoint handlers.

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::app::AppState;

/// Health check response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: DatabaseHealth,
}

/// Database health status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseHealth {
    pub connected: bool,
    pub latency_ms: Option<u64>,
}

/// Simple status response for liveness/readiness probes.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
}

/// Full health check endpoint.
///
/// Returns detailed health information including database connectivity.
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let start = std::time::Instant::now();
    let db_connected = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    let latency_ms = start.elapsed().as_millis() as u64;

    let response = HealthResponse {
        status: if db_connected { "healthy" } else { "unhealthy" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: DatabaseHealth {
            connected: db_connected,
            latency_ms: if db_connected { Some(latency_ms) } else { None },
        },
    };

    if db_connected {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Liveness probe endpoint.
///
/// Returns 200 OK if the process is running.
pub async fn live() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "alive".to_string(),
    })
}

/// Readiness probe endpoint.
///
/// Returns 200 OK if the service can accept traffic (database connected).
pub async fn ready(State(state): State<AppState>) -> Result<Json<StatusResponse>, StatusCode> {
    let db_connected = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();

    if db_connected {
        Ok(Json(StatusResponse {
            status: "ready".to_string(),
        }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
