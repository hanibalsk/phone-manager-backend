//! Admin migration operations API routes.
//!
//! Provides administrative endpoints for querying migration history.
//! Story UGM-2.4: Migration History Query (Admin)

use axum::{
    extract::{Extension, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::api_key::ApiKeyAuth;
use persistence::entities::MigrationStatusDb;
use persistence::repositories::{ListMigrationAuditQuery, MigrationAuditRepository};

/// Query parameters for listing migration history.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListMigrationsQuery {
    /// Filter by user ID
    pub user_id: Option<Uuid>,

    /// Filter by status (success, failed, partial)
    pub status: Option<String>,

    /// Filter by registration group ID
    pub registration_group_id: Option<String>,

    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Items per page (1-100)
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

/// A single migration record in the response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MigrationRecord {
    pub migration_id: Uuid,
    pub user_id: Uuid,
    pub user_email: Option<String>,
    pub registration_group_id: String,
    pub authenticated_group_id: Uuid,
    pub group_name: Option<String>,
    pub devices_migrated: i32,
    pub device_ids: Vec<Uuid>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Response for listing migrations.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListMigrationsResponse {
    pub data: Vec<MigrationRecord>,
    pub pagination: PaginationInfo,
}

/// Pagination information.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PaginationInfo {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// GET /api/admin/v1/migrations
///
/// Returns paginated list of migration history records.
/// Requires admin API key authentication.
pub async fn list_migrations(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuth>,
    Query(query): Query<ListMigrationsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate pagination
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 100);

    // Parse status filter
    let status = query.status.as_ref().and_then(|s| match s.as_str() {
        "success" => Some(MigrationStatusDb::Success),
        "failed" => Some(MigrationStatusDb::Failed),
        "partial" => Some(MigrationStatusDb::Partial),
        _ => None,
    });

    let repo = MigrationAuditRepository::new(state.pool.clone());

    let (records, total) = repo
        .list(ListMigrationAuditQuery {
            user_id: query.user_id,
            status,
            registration_group_id: query.registration_group_id.clone(),
            page,
            per_page,
        })
        .await?;

    // Transform to response format
    let data: Vec<MigrationRecord> = records
        .into_iter()
        .map(|r| MigrationRecord {
            migration_id: r.id,
            user_id: r.user_id,
            user_email: r.user_email,
            registration_group_id: r.registration_group_id,
            authenticated_group_id: r.authenticated_group_id,
            group_name: r.group_name,
            devices_migrated: r.devices_migrated,
            device_ids: r.device_ids,
            status: match r.status {
                MigrationStatusDb::Success => "success".to_string(),
                MigrationStatusDb::Failed => "failed".to_string(),
                MigrationStatusDb::Partial => "partial".to_string(),
            },
            error_message: r.error_message,
            created_at: r.created_at,
        })
        .collect();

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    info!(
        admin_key_id = auth.api_key_id,
        page = page,
        per_page = per_page,
        total = total,
        filters = ?(&query.user_id, &query.status, &query.registration_group_id),
        "Admin queried migration history"
    );

    Ok(Json(ListMigrationsResponse {
        data,
        pagination: PaginationInfo {
            page,
            per_page,
            total,
            total_pages,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_migrations_query_defaults() {
        let json = r#"{}"#;
        let query: ListMigrationsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 20);
        assert!(query.user_id.is_none());
        assert!(query.status.is_none());
    }

    #[test]
    fn test_list_migrations_query_with_filters() {
        let json = r#"{"user_id": "00000000-0000-0000-0000-000000000001", "status": "success", "page": 2, "per_page": 50}"#;
        let query: ListMigrationsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page, 2);
        assert_eq!(query.per_page, 50);
        assert!(query.user_id.is_some());
        assert_eq!(query.status, Some("success".to_string()));
    }

    #[test]
    fn test_migration_record_serialization() {
        let record = MigrationRecord {
            migration_id: Uuid::nil(),
            user_id: Uuid::nil(),
            user_email: Some("test@example.com".to_string()),
            registration_group_id: "camping-2025".to_string(),
            authenticated_group_id: Uuid::nil(),
            group_name: Some("Camping Trip".to_string()),
            devices_migrated: 3,
            device_ids: vec![Uuid::nil()],
            status: "success".to_string(),
            error_message: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"migration_id\""));
        assert!(json.contains("\"devices_migrated\":3"));
        assert!(json.contains("\"status\":\"success\""));
    }

    #[test]
    fn test_pagination_info_serialization() {
        let pagination = PaginationInfo {
            page: 1,
            per_page: 20,
            total: 100,
            total_pages: 5,
        };
        let json = serde_json::to_string(&pagination).unwrap();
        assert!(json.contains("\"page\":1"));
        assert!(json.contains("\"per_page\":20"));
        assert!(json.contains("\"total\":100"));
        assert!(json.contains("\"total_pages\":5"));
    }
}
