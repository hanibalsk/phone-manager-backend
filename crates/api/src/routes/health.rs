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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_healthy() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            database: DatabaseHealth {
                connected: true,
                latency_ms: Some(5),
            },
        };
        assert_eq!(response.status, "healthy");
        assert_eq!(response.version, "0.1.0");
        assert!(response.database.connected);
        assert_eq!(response.database.latency_ms, Some(5));
    }

    #[test]
    fn test_health_response_unhealthy() {
        let response = HealthResponse {
            status: "unhealthy".to_string(),
            version: "0.1.0".to_string(),
            database: DatabaseHealth {
                connected: false,
                latency_ms: None,
            },
        };
        assert_eq!(response.status, "unhealthy");
        assert!(!response.database.connected);
        assert_eq!(response.database.latency_ms, None);
    }

    #[test]
    fn test_database_health_connected() {
        let health = DatabaseHealth {
            connected: true,
            latency_ms: Some(10),
        };
        assert!(health.connected);
        assert_eq!(health.latency_ms, Some(10));
    }

    #[test]
    fn test_database_health_disconnected() {
        let health = DatabaseHealth {
            connected: false,
            latency_ms: None,
        };
        assert!(!health.connected);
        assert!(health.latency_ms.is_none());
    }

    #[test]
    fn test_status_response() {
        let response = StatusResponse {
            status: "alive".to_string(),
        };
        assert_eq!(response.status, "alive");
    }

    #[test]
    fn test_status_response_ready() {
        let response = StatusResponse {
            status: "ready".to_string(),
        };
        assert_eq!(response.status, "ready");
    }
}
