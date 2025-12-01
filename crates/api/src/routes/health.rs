//! Health check endpoint handlers.

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::app::AppState;

/// Health check response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: DatabaseHealth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_services: Option<ExternalServicesHealth>,
}

/// Database health status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DatabaseHealth {
    pub connected: bool,
    pub latency_ms: Option<u64>,
}

/// External services health status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ExternalServicesHealth {
    pub map_matching: MapMatchingHealth,
}

/// Map-matching service health status.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MapMatchingHealth {
    /// Whether map-matching is configured and enabled.
    pub enabled: bool,
    /// Whether the service is currently available (circuit closed).
    pub available: bool,
    /// Current circuit breaker state.
    pub circuit_state: String,
}

/// Simple status response for liveness/readiness probes.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
}

/// Full health check endpoint.
///
/// Returns detailed health information including database connectivity
/// and external service status.
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let start = std::time::Instant::now();
    let db_connected = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    let latency_ms = start.elapsed().as_millis() as u64;

    // Check map-matching service status using the shared client
    let map_matching_config = &state.config.map_matching;
    let map_matching_enabled = map_matching_config.enabled;
    let map_matching_configured = !map_matching_config.url.is_empty();

    // Get actual circuit breaker state from the shared client
    let (map_matching_available, circuit_state) =
        if let Some(ref client) = state.map_matching_client {
            let state = client.circuit_state().await;
            let is_available = client.is_available()
                && matches!(
                    state,
                    crate::services::map_matching::CircuitState::Closed
                        | crate::services::map_matching::CircuitState::HalfOpen
                );
            let state_str = match state {
                crate::services::map_matching::CircuitState::Closed => "closed",
                crate::services::map_matching::CircuitState::Open => "open",
                crate::services::map_matching::CircuitState::HalfOpen => "half_open",
            };
            (is_available, state_str.to_string())
        } else if !map_matching_enabled {
            (false, "disabled".to_string())
        } else if !map_matching_configured {
            (false, "not_configured".to_string())
        } else {
            // Client failed to initialize
            (false, "initialization_failed".to_string())
        };

    let external_services = Some(ExternalServicesHealth {
        map_matching: MapMatchingHealth {
            enabled: map_matching_enabled,
            available: map_matching_available,
            circuit_state,
        },
    });

    let response = HealthResponse {
        status: if db_connected { "healthy" } else { "unhealthy" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: DatabaseHealth {
            connected: db_connected,
            latency_ms: if db_connected { Some(latency_ms) } else { None },
        },
        external_services,
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
            external_services: None,
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
            external_services: None,
        };
        assert_eq!(response.status, "unhealthy");
        assert!(!response.database.connected);
        assert_eq!(response.database.latency_ms, None);
    }

    #[test]
    fn test_health_response_with_external_services() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            database: DatabaseHealth {
                connected: true,
                latency_ms: Some(5),
            },
            external_services: Some(ExternalServicesHealth {
                map_matching: MapMatchingHealth {
                    enabled: true,
                    available: true,
                    circuit_state: "closed".to_string(),
                },
            }),
        };
        assert!(response.external_services.is_some());
        let services = response.external_services.unwrap();
        assert!(services.map_matching.enabled);
        assert!(services.map_matching.available);
        assert_eq!(services.map_matching.circuit_state, "closed");
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

    #[test]
    fn test_map_matching_health_disabled() {
        let health = MapMatchingHealth {
            enabled: false,
            available: false,
            circuit_state: "disabled".to_string(),
        };
        assert!(!health.enabled);
        assert!(!health.available);
        assert_eq!(health.circuit_state, "disabled");
    }

    #[test]
    fn test_map_matching_health_not_configured() {
        let health = MapMatchingHealth {
            enabled: true,
            available: false,
            circuit_state: "not_configured".to_string(),
        };
        assert!(health.enabled);
        assert!(!health.available);
        assert_eq!(health.circuit_state, "not_configured");
    }

    #[test]
    fn test_map_matching_health_closed() {
        let health = MapMatchingHealth {
            enabled: true,
            available: true,
            circuit_state: "closed".to_string(),
        };
        assert!(health.enabled);
        assert!(health.available);
        assert_eq!(health.circuit_state, "closed");
    }

    #[test]
    fn test_map_matching_health_open() {
        let health = MapMatchingHealth {
            enabled: true,
            available: false,
            circuit_state: "open".to_string(),
        };
        assert!(health.enabled);
        assert!(!health.available);
        assert_eq!(health.circuit_state, "open");
    }

    #[test]
    fn test_map_matching_health_half_open() {
        let health = MapMatchingHealth {
            enabled: true,
            available: true,
            circuit_state: "half_open".to_string(),
        };
        assert!(health.enabled);
        assert!(health.available);
        assert_eq!(health.circuit_state, "half_open");
    }

    #[test]
    fn test_external_services_health_serialization() {
        let health = ExternalServicesHealth {
            map_matching: MapMatchingHealth {
                enabled: true,
                available: true,
                circuit_state: "closed".to_string(),
            },
        };
        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("\"map_matching\""));
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"available\":true"));
        assert!(json.contains("\"circuit_state\":\"closed\""));
    }
}
