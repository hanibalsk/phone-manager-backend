//! Feature toggle middleware for enabling/disabling optional modules.
//!
//! These middleware functions check configuration to determine if a feature
//! is enabled. When disabled, they return 404 "Feature not available".

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::app::AppState;

/// Helper to create a feature disabled response (404).
fn feature_disabled_response(feature_name: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "feature_disabled",
            "message": format!("{} feature is not available", feature_name)
        })),
    )
        .into_response()
}

/// Middleware that checks if geofences feature is enabled.
///
/// When `features.geofences_enabled` is false, returns 404.
pub async fn require_geofences(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !state.config.features.geofences_enabled {
        return feature_disabled_response("Geofences");
    }
    next.run(req).await
}

/// Middleware that checks if proximity alerts feature is enabled.
///
/// When `features.proximity_alerts_enabled` is false, returns 404.
pub async fn require_proximity_alerts(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !state.config.features.proximity_alerts_enabled {
        return feature_disabled_response("Proximity alerts");
    }
    next.run(req).await
}

/// Middleware that checks if webhooks feature is enabled.
///
/// When `features.webhooks_enabled` is false, returns 404.
pub async fn require_webhooks(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !state.config.features.webhooks_enabled {
        return feature_disabled_response("Webhooks");
    }
    next.run(req).await
}

/// Middleware that checks if movement tracking feature is enabled.
///
/// When `features.movement_tracking_enabled` is false, returns 404.
/// Affects trips and movement events endpoints.
pub async fn require_movement_tracking(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !state.config.features.movement_tracking_enabled {
        return feature_disabled_response("Movement tracking");
    }
    next.run(req).await
}

/// Middleware that checks if B2B/organization features are enabled.
///
/// When `features.b2b_enabled` is false, returns 404.
/// Affects organizations, device policies, enrollment tokens, fleet management.
pub async fn require_b2b(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !state.config.features.b2b_enabled {
        return feature_disabled_response("B2B/Organization");
    }
    next.run(req).await
}

/// Middleware that checks if geofence events feature is enabled.
///
/// When `features.geofence_events_enabled` is false, returns 404.
pub async fn require_geofence_events(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if !state.config.features.geofence_events_enabled {
        return feature_disabled_response("Geofence events");
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_disabled_response() {
        let response = feature_disabled_response("Test Feature");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
