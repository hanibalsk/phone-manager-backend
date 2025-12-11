//! Audit log routes.
//!
//! Story 13.9: Audit Logging System
//! Story 13.10: Audit Query and Export Endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::{
    AsyncExportResponse, AuditLog, AuditLogPagination, ExportAuditLogsQuery, ExportFormat,
    ExportJobResponse, ExportJobStatus, ListAuditLogsQuery, ListAuditLogsResponse,
    SyncExportResponse, MAX_EXPORT_RECORDS, MAX_SYNC_EXPORT_RECORDS,
};
use persistence::repositories::{AuditExportJobRepository, AuditLogRepository};

/// Create audit logs router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_audit_logs))
        .route("/export", get(export_audit_logs))
        .route("/export/:job_id", get(get_export_job_status))
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

/// Export audit logs as CSV or JSON.
/// For small datasets (<= 1000 records), returns the data directly.
/// For larger datasets, creates an async job and returns the job ID.
/// Rate limited to 10 exports per hour per organization.
#[axum::debug_handler]
pub async fn export_audit_logs(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ExportAuditLogsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // Check export rate limit (10/hour/org)
    if let Some(ref export_limiter) = state.export_rate_limiter {
        if let Err(retry_after) = export_limiter.check(org_id) {
            return Err(ApiError::RateLimitedWithRetry {
                message: format!(
                    "Export rate limit of {} exports/hour exceeded for this organization",
                    export_limiter.rate_limit_per_hour()
                ),
                retry_after,
            });
        }
    }

    let log_repo = AuditLogRepository::new(state.pool.clone());
    let job_repo = AuditExportJobRepository::new(state.pool.clone());

    let format = query.format.unwrap_or_default();
    let list_query = query.to_list_query();

    // First, count the records to determine sync vs async export
    let (_, total) = log_repo.list(org_id, &list_query).await?;

    if total > MAX_EXPORT_RECORDS {
        return Err(ApiError::Validation(format!(
            "Export limited to {} records. Use filters to reduce result set.",
            MAX_EXPORT_RECORDS
        )));
    }

    if total <= MAX_SYNC_EXPORT_RECORDS {
        // Sync export: fetch all and return directly
        let logs = log_repo
            .list_for_export(org_id, &list_query, MAX_SYNC_EXPORT_RECORDS)
            .await?;
        let (data, content_type) = generate_export_data(&logs, format)?;

        // Create data URL
        let download_url = format!("data:{};base64,{}", content_type, STANDARD.encode(&data));

        let response = SyncExportResponse {
            format,
            record_count: logs.len() as i64,
            download_url,
        };

        return Ok((StatusCode::OK, Json(ExportResponse::Sync(response))));
    }

    // Async export: create job and process in background
    let filters = serde_json::to_value(&query).ok();
    let job = job_repo.create(org_id, format, filters).await?;

    // Spawn background task to process the export
    let pool = state.pool.clone();
    let job_id = job.job_id.clone();
    tokio::spawn(async move {
        process_export_job(pool, org_id, job_id, list_query, format).await;
    });

    let response = AsyncExportResponse {
        job_id: job.job_id.clone(),
        status: ExportJobStatus::Processing,
        estimated_records: total,
        check_url: format!(
            "/api/admin/v1/organizations/{}/audit-logs/export/{}",
            org_id, job.job_id
        ),
    };

    Ok((StatusCode::ACCEPTED, Json(ExportResponse::Async(response))))
}

/// Get export job status.
#[axum::debug_handler]
pub async fn get_export_job_status(
    State(state): State<AppState>,
    Path((org_id, job_id)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let job_repo = AuditExportJobRepository::new(state.pool.clone());

    let job = job_repo.find_by_job_id(org_id, &job_id).await?;

    match job {
        Some(job) => {
            let response = ExportJobResponse {
                job_id: job.job_id,
                status: job.status,
                record_count: job.record_count,
                download_url: job.download_url,
                expires_at: Some(job.expires_at),
                error: job.error_message,
            };
            Ok((StatusCode::OK, Json(response)))
        }
        None => Err(ApiError::NotFound("Export job not found".to_string())),
    }
}

/// Background task to process export job.
async fn process_export_job(
    pool: sqlx::PgPool,
    org_id: Uuid,
    job_id: String,
    query: ListAuditLogsQuery,
    format: ExportFormat,
) {
    let log_repo = AuditLogRepository::new(pool.clone());
    let job_repo = AuditExportJobRepository::new(pool);

    // Mark job as processing
    if let Err(e) = job_repo.mark_processing(&job_id).await {
        tracing::error!("Failed to mark job as processing: {}", e);
        return;
    }

    // Fetch the data
    match log_repo
        .list_for_export(org_id, &query, MAX_EXPORT_RECORDS)
        .await
    {
        Ok(logs) => {
            match generate_export_data(&logs, format) {
                Ok((data, content_type)) => {
                    // Create data URL
                    let download_url =
                        format!("data:{};base64,{}", content_type, STANDARD.encode(&data));

                    if let Err(e) = job_repo
                        .mark_completed(&job_id, logs.len() as i64, &download_url)
                        .await
                    {
                        tracing::error!("Failed to mark job as completed: {}", e);
                    }
                }
                Err(e) => {
                    if let Err(e2) = job_repo.mark_failed(&job_id, &e.to_string()).await {
                        tracing::error!("Failed to mark job as failed: {}", e2);
                    }
                }
            }
        }
        Err(e) => {
            if let Err(e2) = job_repo.mark_failed(&job_id, &e.to_string()).await {
                tracing::error!("Failed to mark job as failed: {}", e2);
            }
        }
    }
}

/// Generate export data in the specified format.
fn generate_export_data(
    logs: &[AuditLog],
    format: ExportFormat,
) -> Result<(Vec<u8>, &'static str), ApiError> {
    match format {
        ExportFormat::Json => {
            let json =
                serde_json::to_vec_pretty(logs).map_err(|e| ApiError::Internal(e.to_string()))?;
            Ok((json, "application/json"))
        }
        ExportFormat::Csv => {
            let csv = generate_csv(logs)?;
            Ok((csv.into_bytes(), "text/csv"))
        }
    }
}

/// Generate CSV from audit logs.
/// Includes UTF-8 BOM for Excel compatibility.
fn generate_csv(logs: &[AuditLog]) -> Result<String, ApiError> {
    let mut csv = String::new();

    // Add UTF-8 BOM for Excel compatibility
    csv.push('\u{FEFF}');

    // Header
    csv.push_str("id,timestamp,actor_type,actor_id,actor_email,action,resource_type,resource_id,resource_name,ip_address,user_agent\n");

    for log in logs {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            log.id,
            log.timestamp.to_rfc3339(),
            log.actor.actor_type,
            log.actor.id.map(|u| u.to_string()).unwrap_or_default(),
            escape_csv(log.actor.email.as_deref().unwrap_or("")),
            escape_csv(&log.action),
            escape_csv(&log.resource.resource_type),
            escape_csv(log.resource.id.as_deref().unwrap_or("")),
            escape_csv(log.resource.name.as_deref().unwrap_or("")),
            log.metadata
                .as_ref()
                .and_then(|m| m.ip_address.as_deref())
                .unwrap_or(""),
            escape_csv(
                log.metadata
                    .as_ref()
                    .and_then(|m| m.user_agent.as_deref())
                    .unwrap_or("")
            )
        ));
    }

    Ok(csv)
}

/// Escape a value for CSV output.
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Combined response type for export endpoint.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
enum ExportResponse {
    Sync(SyncExportResponse),
    Async(AsyncExportResponse),
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

    #[test]
    fn test_escape_csv_simple() {
        assert_eq!(escape_csv("hello"), "hello");
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
        assert_eq!(escape_csv("hello\"world"), "\"hello\"\"world\"");
    }

    #[test]
    fn test_escape_csv_with_newline() {
        assert_eq!(escape_csv("hello\nworld"), "\"hello\nworld\"");
    }

    #[test]
    fn test_generate_csv_has_bom() {
        let csv = generate_csv(&[]).unwrap();
        // UTF-8 BOM is U+FEFF
        assert!(csv.starts_with('\u{FEFF}'));
        // Should contain header after BOM
        assert!(csv.contains("id,timestamp,actor_type"));
    }

    #[test]
    fn test_export_query_to_list_query() {
        let export_query = ExportAuditLogsQuery {
            format: Some(ExportFormat::Csv),
            actor_id: Some(Uuid::new_v4()),
            action: Some("device.assign".to_string()),
            resource_type: None,
            resource_id: None,
            from: None,
            to: None,
        };

        let list_query = export_query.to_list_query();
        assert_eq!(list_query.actor_id, export_query.actor_id);
        assert_eq!(list_query.action, export_query.action);
    }
}
