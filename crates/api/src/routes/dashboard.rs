//! Dashboard metrics routes.
//!
//! Story 14.1: Dashboard Metrics Endpoint

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;
use persistence::repositories::{DashboardRepository, OrganizationRepository};

/// GET /api/admin/v1/organizations/{org_id}/dashboard
///
/// Get dashboard metrics for an organization.
pub async fn get_dashboard_metrics(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let org_repo = OrganizationRepository::new(state.pool.clone());
    let dashboard_repo = DashboardRepository::new(state.pool.clone());

    // Verify organization exists
    let org = org_repo.find_by_id(org_id).await?;
    if org.is_none() {
        return Err(ApiError::NotFound("Organization not found".to_string()));
    }

    // Get dashboard metrics
    let metrics = dashboard_repo.get_metrics(org_id).await?;

    info!(
        admin_key_id = auth.api_key_id,
        organization_id = %org_id,
        device_count = metrics.devices.total,
        user_count = metrics.users.total,
        "Fetched dashboard metrics"
    );

    Ok((StatusCode::OK, Json(metrics)))
}

#[cfg(test)]
mod tests {
    use domain::models::DashboardMetrics;

    #[test]
    fn test_dashboard_metrics_response_serialization() {
        let metrics = DashboardMetrics::new();
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("devices"));
        assert!(json.contains("users"));
        assert!(json.contains("generated_at"));
    }
}
