//! Audit log routes.
//!
//! Story 13.9: Audit Logging System
//! Story 13.10: Audit Query and Export Endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
    routing::get,
};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::{
    AuditLogPagination, ListAuditLogsQuery, ListAuditLogsResponse,
};
use persistence::repositories::AuditLogRepository;

/// Create audit logs router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_audit_logs))
        .route("/:log_id", get(get_audit_log))
}

/// List audit logs with filtering and pagination.
#[axum::debug_handler]
pub async fn list_audit_logs(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListAuditLogsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = AuditLogRepository::new(state.pool.clone());

    let (logs, total) = repo.list(org_id, &query).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    let response = ListAuditLogsResponse {
        data: logs,
        pagination: AuditLogPagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get a specific audit log entry.
#[axum::debug_handler]
pub async fn get_audit_log(
    State(state): State<AppState>,
    Path((org_id, log_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = AuditLogRepository::new(state.pool.clone());

    let log = repo.find_by_id(org_id, log_id).await?;

    match log {
        Some(log) => Ok((StatusCode::OK, Json(log))),
        None => Err(ApiError::NotFound("Audit log not found".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_audit_logs_query_defaults() {
        let query = ListAuditLogsQuery::default();
        assert!(query.page.is_none());
        assert!(query.per_page.is_none());
    }
}
