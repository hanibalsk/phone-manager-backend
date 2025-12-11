//! Analytics and reporting routes.
//!
//! AP-10: Dashboard & Analytics - User, Device, and API analytics

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::header,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use chrono::{Datelike, Duration, NaiveDate, Utc};
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;
use domain::models::{
    AnalyticsDeviceStatusBreakdown, AnalyticsGroupBy, AnalyticsPeriod, ApiUsageAnalyticsQuery,
    ApiUsageAnalyticsResponse, ApiUsageSummary, ApiUsageTrend, DeviceActivityTrend,
    DeviceAnalyticsQuery, DeviceAnalyticsResponse, DeviceAnalyticsSummary, EndpointUsage,
    GenerateReportRequest, OrgUserRole, ReportJobResponse, ReportStatus,
    UserActivityTrend, UserAnalyticsQuery, UserAnalyticsResponse, UserAnalyticsSummary,
    UserRoleBreakdown,
};
use persistence::repositories::{AnalyticsRepository, OrgUserRepository};

/// Build the analytics router.
pub fn router() -> Router<AppState> {
    Router::new()
        // Analytics endpoints (FR-10.1, FR-10.2, FR-10.3)
        .route("/users", get(get_user_analytics))
        .route("/devices", get(get_device_analytics))
        .route("/api", get(get_api_usage_analytics))
}

/// Build the reports router.
pub fn reports_router() -> Router<AppState> {
    Router::new()
        // Report generation endpoints (FR-10.4, FR-10.5, FR-10.6, FR-10.7)
        .route("/users", post(generate_user_report))
        .route("/devices", post(generate_device_report))
        .route("/:report_id/status", get(get_report_status))
        .route("/:report_id/download", get(download_report))
}

/// Helper function to verify org admin access.
async fn verify_org_admin(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), ApiError> {
    let org_user_repo = OrgUserRepository::new(pool.clone());
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Only owners and admins can view analytics
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden("Insufficient permissions".to_string()));
    }

    Ok(())
}

/// Get user analytics for organization (FR-10.1).
#[axum::debug_handler]
async fn get_user_analytics(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<UserAnalyticsQuery>,
    user: UserAuth,
) -> Result<Json<UserAnalyticsResponse>, ApiError> {
    // Verify user has admin access to organization
    verify_org_admin(&state.pool, org_id, user.user_id).await?;

    let repo = AnalyticsRepository::new(state.pool.clone());

    // Default to last 30 days if not specified
    let today = Utc::now().date_naive();
    let from = query.from.unwrap_or_else(|| today - Duration::days(30));
    let to = query.to.unwrap_or(today);

    // Get user analytics summary
    let summary_entity = repo.get_user_analytics_summary(org_id, from, to).await?;

    // Get user activity trends
    let trends_entities = repo.get_user_activity_trends(org_id, from, to).await?;

    // Get user role breakdown
    let role_entities = repo.get_user_role_breakdown(org_id).await?;

    // Convert entities to domain models
    let total_sessions = summary_entity.total_sessions;
    let active_users = summary_entity.active_users;
    let avg_sessions_per_user = if active_users > 0 {
        total_sessions as f64 / active_users as f64
    } else {
        0.0
    };

    let summary = UserAnalyticsSummary {
        total_users: summary_entity.total_users,
        active_users: summary_entity.active_users,
        new_users_period: summary_entity.new_users_period,
        avg_sessions_per_user,
        avg_session_duration_seconds: summary_entity.avg_session_duration,
    };

    let trends: Vec<UserActivityTrend> = trends_entities
        .into_iter()
        .map(|e| UserActivityTrend {
            date: e.activity_date,
            active_users: e.active_users,
            new_users: e.new_users,
            returning_users: e.returning_users,
            total_sessions: e.total_sessions,
        })
        .collect();

    // Aggregate trends by group_by if specified
    let trends = aggregate_user_trends(trends, query.group_by);

    // Parse role breakdown
    let mut by_role = UserRoleBreakdown {
        owners: 0,
        admins: 0,
        members: 0,
    };
    for role in role_entities {
        match role.role.as_str() {
            "owner" => by_role.owners = role.count,
            "admin" => by_role.admins = role.count,
            "member" => by_role.members = role.count,
            _ => by_role.members += role.count, // Default to members
        }
    }

    let response = UserAnalyticsResponse {
        organization_id: org_id,
        period: AnalyticsPeriod { start: from, end: to },
        summary,
        trends,
        by_role,
    };

    Ok(Json(response))
}

/// Get device analytics for organization (FR-10.2).
#[axum::debug_handler]
async fn get_device_analytics(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<DeviceAnalyticsQuery>,
    user: UserAuth,
) -> Result<Json<DeviceAnalyticsResponse>, ApiError> {
    // Verify user has admin access to organization
    verify_org_admin(&state.pool, org_id, user.user_id).await?;

    let repo = AnalyticsRepository::new(state.pool.clone());

    // Default to last 30 days if not specified
    let today = Utc::now().date_naive();
    let from = query.from.unwrap_or_else(|| today - Duration::days(30));
    let to = query.to.unwrap_or(today);

    // Get device analytics summary
    let summary_entity = repo.get_device_analytics_summary(org_id, from, to).await?;

    // Get device activity trends
    let trends_entities = repo.get_device_activity_trends(org_id, from, to).await?;

    // Get device status breakdown
    let status_entities = repo.get_device_status_breakdown(org_id).await?;

    // Convert entities to domain models
    let summary = DeviceAnalyticsSummary {
        total_devices: summary_entity.total_devices,
        active_devices: summary_entity.active_devices,
        new_enrollments_period: summary_entity.new_enrollments,
        unenrollments_period: summary_entity.unenrollments,
        total_locations_reported: summary_entity.total_locations,
        total_geofence_events: summary_entity.total_geofence_events,
        total_commands_issued: summary_entity.total_commands,
    };

    let trends: Vec<DeviceActivityTrend> = trends_entities
        .into_iter()
        .map(|e| DeviceActivityTrend {
            date: e.activity_date,
            active_devices: e.active_devices,
            new_enrollments: e.new_enrollments,
            unenrollments: e.unenrollments,
            locations_reported: e.total_locations_reported,
            geofence_events: e.total_geofence_events,
        })
        .collect();

    // Aggregate trends by group_by if specified
    let trends = aggregate_device_trends(trends, query.group_by);

    // Parse status breakdown
    let mut by_status = AnalyticsDeviceStatusBreakdown {
        registered: 0,
        enrolled: 0,
        suspended: 0,
        retired: 0,
    };
    for status in status_entities {
        match status.status.as_str() {
            "registered" => by_status.registered = status.count,
            "enrolled" => by_status.enrolled = status.count,
            "suspended" => by_status.suspended = status.count,
            "retired" => by_status.retired = status.count,
            _ => {} // Ignore unknown statuses
        }
    }

    let response = DeviceAnalyticsResponse {
        organization_id: org_id,
        period: AnalyticsPeriod { start: from, end: to },
        summary,
        trends,
        by_status,
    };

    Ok(Json(response))
}

/// Get API usage analytics for organization (FR-10.3).
#[axum::debug_handler]
async fn get_api_usage_analytics(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ApiUsageAnalyticsQuery>,
    user: UserAuth,
) -> Result<Json<ApiUsageAnalyticsResponse>, ApiError> {
    // Verify user has admin access to organization
    verify_org_admin(&state.pool, org_id, user.user_id).await?;

    let repo = AnalyticsRepository::new(state.pool.clone());

    // Default to last 30 days if not specified
    let today = Utc::now().date_naive();
    let from = query.from.unwrap_or_else(|| today - Duration::days(30));
    let to = query.to.unwrap_or(today);

    // Get API usage summary
    let summary_entity = repo.get_api_usage_summary(org_id, from, to).await?;

    // Get API usage trends
    let trends_entities = repo.get_api_usage_trends(org_id, from, to).await?;

    // Get top endpoints
    let top_endpoints_entities = repo.get_top_endpoints(org_id, from, to, 10).await?;

    // Convert entities to domain models
    let total_requests = summary_entity.total_requests;
    let success_rate = if total_requests > 0 {
        summary_entity.success_count as f64 / total_requests as f64 * 100.0
    } else {
        100.0
    };

    let summary = ApiUsageSummary {
        total_requests: summary_entity.total_requests,
        success_count: summary_entity.success_count,
        error_count: summary_entity.error_count,
        success_rate,
        avg_response_time_ms: summary_entity.avg_response_time_ms,
        p95_response_time_ms: summary_entity.p95_response_time_ms,
        total_data_transferred_bytes: summary_entity.total_bytes,
    };

    // Aggregate trends by date (since they're per-endpoint in the DB)
    let mut trends_map: std::collections::HashMap<NaiveDate, ApiUsageTrend> =
        std::collections::HashMap::new();
    for e in trends_entities {
        let entry = trends_map.entry(e.usage_date).or_insert(ApiUsageTrend {
            date: e.usage_date,
            total_requests: 0,
            success_count: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
        });
        entry.total_requests += e.total_requests;
        entry.success_count += e.success_count;
        entry.error_count += e.error_count;
        // Simple average of averages (not weighted, but good enough for trends)
        if entry.avg_response_time_ms == 0.0 {
            entry.avg_response_time_ms = e.avg_response_time_ms.unwrap_or(0.0);
        } else {
            entry.avg_response_time_ms =
                (entry.avg_response_time_ms + e.avg_response_time_ms.unwrap_or(0.0)) / 2.0;
        }
    }
    let mut trends: Vec<ApiUsageTrend> = trends_map.into_values().collect();
    trends.sort_by(|a, b| a.date.cmp(&b.date));

    // Aggregate trends by group_by if specified
    let trends = aggregate_api_trends(trends, query.group_by);

    // Convert top endpoints
    let top_endpoints: Vec<EndpointUsage> = top_endpoints_entities
        .into_iter()
        .map(|e| {
            let success_rate = if e.total_requests > 0 {
                e.success_count as f64 / e.total_requests as f64 * 100.0
            } else {
                100.0
            };
            let percentage = if total_requests > 0 {
                e.total_requests as f64 / total_requests as f64 * 100.0
            } else {
                0.0
            };
            EndpointUsage {
                endpoint: e.endpoint_path,
                method: e.method,
                total_requests: e.total_requests,
                success_rate,
                avg_response_time_ms: e.avg_response_time_ms,
                percentage,
            }
        })
        .collect();

    let response = ApiUsageAnalyticsResponse {
        organization_id: org_id,
        period: AnalyticsPeriod { start: from, end: to },
        summary,
        trends,
        top_endpoints,
    };

    Ok(Json(response))
}

/// Generate user report (FR-10.4).
#[axum::debug_handler]
async fn generate_user_report(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    user: UserAuth,
    Json(request): Json<GenerateReportRequest>,
) -> Result<Json<ReportJobResponse>, ApiError> {
    // Verify user has admin access to organization
    verify_org_admin(&state.pool, org_id, user.user_id).await?;

    let repo = AnalyticsRepository::new(state.pool.clone());

    let parameters = serde_json::json!({
        "from": request.from,
        "to": request.to,
        "format": request.format,
        "additional": request.parameters,
    });

    let job = repo
        .create_report_job(org_id, "user_analytics", parameters, user.user_id)
        .await?;

    // TODO(FR-10.5-10.9): Report Generation Background Worker
    // ============================================================
    // This is a stub implementation. A background worker is needed to:
    //
    // 1. Process report jobs asynchronously (FR-10.5)
    //    - Monitor `report_jobs` table for pending jobs
    //    - Update job status: pending -> processing -> completed/failed
    //    - Set started_at when processing begins
    //    - Set completed_at when processing finishes
    //
    // 2. Generate actual report content (FR-10.5)
    //    - Query user analytics data from AnalyticsRepository
    //    - Aggregate metrics: active users, sessions, activity patterns
    //    - Apply date range filters from job parameters
    //
    // 3. Export to requested format (FR-10.6, FR-10.7)
    //    - Support CSV format with proper headers and escaping
    //    - Support JSON format with structured data
    //    - Support PDF format for printable reports
    //    - Store generated files in configurable storage (local/S3)
    //
    // 4. Handle report lifecycle (FR-10.8, FR-10.9)
    //    - Set file_size_bytes after generation
    //    - Set expires_at (e.g., 7 days from completion)
    //    - Implement cleanup job to delete expired reports
    //    - Store download URL or file path for retrieval
    //
    // Implementation approach:
    // - Create a ReportGenerationWorker similar to WebhookRetryWorker
    // - Run on a configurable interval (e.g., every 30 seconds)
    // - Use tokio::spawn for async processing
    // - Implement proper error handling and retry logic
    // ============================================================

    let response = ReportJobResponse {
        id: job.id,
        organization_id: job.organization_id,
        report_type: job.report_type,
        status: ReportStatus::from(job.status.as_str()),
        parameters: job.parameters,
        file_size_bytes: job.file_size_bytes,
        error_message: job.error_message,
        created_by: job.created_by,
        started_at: job.started_at,
        completed_at: job.completed_at,
        expires_at: job.expires_at,
        created_at: job.created_at,
    };

    Ok(Json(response))
}

/// Generate device report (FR-10.5).
#[axum::debug_handler]
async fn generate_device_report(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    user: UserAuth,
    Json(request): Json<GenerateReportRequest>,
) -> Result<Json<ReportJobResponse>, ApiError> {
    // Verify user has admin access to organization
    verify_org_admin(&state.pool, org_id, user.user_id).await?;

    let repo = AnalyticsRepository::new(state.pool.clone());

    let parameters = serde_json::json!({
        "from": request.from,
        "to": request.to,
        "format": request.format,
        "additional": request.parameters,
    });

    let job = repo
        .create_report_job(org_id, "device_analytics", parameters, user.user_id)
        .await?;

    // TODO(FR-10.5-10.9): Report Generation Background Worker
    // See detailed implementation notes in generate_user_report above.
    // This handler creates a device analytics report job that includes:
    // - Device enrollment statistics
    // - Device activity and status distribution
    // - Policy compliance metrics
    // - Device health and connectivity patterns

    let response = ReportJobResponse {
        id: job.id,
        organization_id: job.organization_id,
        report_type: job.report_type,
        status: ReportStatus::from(job.status.as_str()),
        parameters: job.parameters,
        file_size_bytes: job.file_size_bytes,
        error_message: job.error_message,
        created_by: job.created_by,
        started_at: job.started_at,
        completed_at: job.completed_at,
        expires_at: job.expires_at,
        created_at: job.created_at,
    };

    Ok(Json(response))
}

/// Get report status (FR-10.6).
#[axum::debug_handler]
async fn get_report_status(
    State(state): State<AppState>,
    Path((org_id, report_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<Json<ReportJobResponse>, ApiError> {
    // Verify user has admin access to organization
    verify_org_admin(&state.pool, org_id, user.user_id).await?;

    let repo = AnalyticsRepository::new(state.pool.clone());

    let job = repo
        .get_report_job(org_id, report_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Report not found".to_string()))?;

    let response = ReportJobResponse {
        id: job.id,
        organization_id: job.organization_id,
        report_type: job.report_type,
        status: ReportStatus::from(job.status.as_str()),
        parameters: job.parameters,
        file_size_bytes: job.file_size_bytes,
        error_message: job.error_message,
        created_by: job.created_by,
        started_at: job.started_at,
        completed_at: job.completed_at,
        expires_at: job.expires_at,
        created_at: job.created_at,
    };

    Ok(Json(response))
}

/// Download report (FR-10.7).
///
/// Streams the actual report file to the client with appropriate headers.
#[axum::debug_handler]
async fn download_report(
    State(state): State<AppState>,
    Path((org_id, report_id)): Path<(Uuid, Uuid)>,
    user: UserAuth,
) -> Result<Response, ApiError> {
    // Verify user has admin access to organization
    verify_org_admin(&state.pool, org_id, user.user_id).await?;

    let repo = AnalyticsRepository::new(state.pool.clone());

    let job = repo
        .get_report_job(org_id, report_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Report not found".to_string()))?;

    // Check if report is completed
    if job.status != "completed" {
        return Err(ApiError::Validation(format!(
            "Report is not ready for download. Status: {}",
            job.status
        )));
    }

    // Check if file path exists in database
    let file_name = job
        .file_path
        .ok_or_else(|| ApiError::NotFound("Report file not found".to_string()))?;

    // Build full path using config
    let reports_dir = std::path::PathBuf::from(&state.config.reports.reports_dir);
    let full_path = reports_dir.join(&file_name);

    // Open the file
    let file = File::open(&full_path).await.map_err(|e| {
        tracing::error!(
            error = %e,
            path = %full_path.display(),
            "Failed to open report file"
        );
        ApiError::NotFound("Report file not found on disk".to_string())
    })?;

    // Get file metadata for content-length
    let metadata = file.metadata().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get file metadata");
        ApiError::Internal("Failed to read report file".to_string())
    })?;

    // Determine content type based on file extension
    let content_type = if file_name.ends_with(".csv") {
        "text/csv"
    } else if file_name.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    };

    // Create a friendly download filename
    let download_filename = format!(
        "{}_{}.{}",
        job.report_type,
        job.created_at.format("%Y%m%d_%H%M%S"),
        if file_name.ends_with(".csv") {
            "csv"
        } else {
            "json"
        }
    );

    // Stream the file
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Build response with appropriate headers
    let response = Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", download_filename),
        )
        .header(header::CONTENT_LENGTH, metadata.len())
        .body(body)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to build response");
            ApiError::Internal("Failed to build response".to_string())
        })?;

    Ok(response)
}

// Helper functions for trend aggregation

fn aggregate_user_trends(
    trends: Vec<UserActivityTrend>,
    group_by: Option<AnalyticsGroupBy>,
) -> Vec<UserActivityTrend> {
    let group_by = group_by.unwrap_or_default();
    if group_by == AnalyticsGroupBy::Day {
        return trends;
    }

    let mut aggregated: std::collections::HashMap<NaiveDate, UserActivityTrend> =
        std::collections::HashMap::new();

    for trend in trends {
        let key = match group_by {
            AnalyticsGroupBy::Week => {
                // Start of ISO week
                trend.date - Duration::days(trend.date.weekday().num_days_from_monday() as i64)
            }
            AnalyticsGroupBy::Month => {
                // First day of month
                NaiveDate::from_ymd_opt(trend.date.year(), trend.date.month(), 1)
                    .unwrap_or(trend.date)
            }
            AnalyticsGroupBy::Day => trend.date,
        };

        let entry = aggregated.entry(key).or_insert(UserActivityTrend {
            date: key,
            active_users: 0,
            new_users: 0,
            returning_users: 0,
            total_sessions: 0,
        });
        entry.active_users += trend.active_users;
        entry.new_users += trend.new_users;
        entry.returning_users += trend.returning_users;
        entry.total_sessions += trend.total_sessions;
    }

    let mut result: Vec<UserActivityTrend> = aggregated.into_values().collect();
    result.sort_by(|a, b| a.date.cmp(&b.date));
    result
}

fn aggregate_device_trends(
    trends: Vec<DeviceActivityTrend>,
    group_by: Option<AnalyticsGroupBy>,
) -> Vec<DeviceActivityTrend> {
    let group_by = group_by.unwrap_or_default();
    if group_by == AnalyticsGroupBy::Day {
        return trends;
    }

    let mut aggregated: std::collections::HashMap<NaiveDate, DeviceActivityTrend> =
        std::collections::HashMap::new();

    for trend in trends {
        let key = match group_by {
            AnalyticsGroupBy::Week => {
                trend.date - Duration::days(trend.date.weekday().num_days_from_monday() as i64)
            }
            AnalyticsGroupBy::Month => {
                NaiveDate::from_ymd_opt(trend.date.year(), trend.date.month(), 1)
                    .unwrap_or(trend.date)
            }
            AnalyticsGroupBy::Day => trend.date,
        };

        let entry = aggregated.entry(key).or_insert(DeviceActivityTrend {
            date: key,
            active_devices: 0,
            new_enrollments: 0,
            unenrollments: 0,
            locations_reported: 0,
            geofence_events: 0,
        });
        entry.active_devices += trend.active_devices;
        entry.new_enrollments += trend.new_enrollments;
        entry.unenrollments += trend.unenrollments;
        entry.locations_reported += trend.locations_reported;
        entry.geofence_events += trend.geofence_events;
    }

    let mut result: Vec<DeviceActivityTrend> = aggregated.into_values().collect();
    result.sort_by(|a, b| a.date.cmp(&b.date));
    result
}

fn aggregate_api_trends(
    trends: Vec<ApiUsageTrend>,
    group_by: Option<AnalyticsGroupBy>,
) -> Vec<ApiUsageTrend> {
    let group_by = group_by.unwrap_or_default();
    if group_by == AnalyticsGroupBy::Day {
        return trends;
    }

    let mut aggregated: std::collections::HashMap<NaiveDate, (ApiUsageTrend, i32)> =
        std::collections::HashMap::new();

    for trend in trends {
        let key = match group_by {
            AnalyticsGroupBy::Week => {
                trend.date - Duration::days(trend.date.weekday().num_days_from_monday() as i64)
            }
            AnalyticsGroupBy::Month => {
                NaiveDate::from_ymd_opt(trend.date.year(), trend.date.month(), 1)
                    .unwrap_or(trend.date)
            }
            AnalyticsGroupBy::Day => trend.date,
        };

        let entry = aggregated.entry(key).or_insert((
            ApiUsageTrend {
                date: key,
                total_requests: 0,
                success_count: 0,
                error_count: 0,
                avg_response_time_ms: 0.0,
            },
            0,
        ));
        entry.0.total_requests += trend.total_requests;
        entry.0.success_count += trend.success_count;
        entry.0.error_count += trend.error_count;
        entry.0.avg_response_time_ms += trend.avg_response_time_ms;
        entry.1 += 1;
    }

    let mut result: Vec<ApiUsageTrend> = aggregated
        .into_values()
        .map(|(mut trend, count)| {
            if count > 0 {
                trend.avg_response_time_ms /= count as f64;
            }
            trend
        })
        .collect();
    result.sort_by(|a, b| a.date.cmp(&b.date));
    result
}
