//! App usage route handlers.
//!
//! AP-8.1-8.2, AP-8.7: App usage summary, history, and analytics endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{Days, Utc};
use tracing::info;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;
use domain::models::{
    AnalyticsSummary, AnalyticsTrendPoint, AppUsageAnalyticsQuery, AppUsageAnalyticsResponse,
    AppUsageHistoryEntry, AppUsageHistoryQuery, AppUsageHistoryResponse, AppUsageItem,
    AppUsagePagination, AppUsagePeriod, AppUsageSummaryQuery, AppUsageSummaryResponse,
    CategoryUsageItem, OrgUserRole, TopAppItem,
};
use persistence::repositories::{AppUsageRepository, DeviceRepository, OrgUserRepository};

/// Create app usage router for device-level endpoints.
///
/// Routes:
/// - GET /api/admin/v1/organizations/:org_id/devices/:device_id/app-usage - Get usage summary
/// - GET /api/admin/v1/organizations/:org_id/devices/:device_id/app-usage/history - Get usage history
pub fn device_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_device_app_usage_summary))
        .route("/history", get(get_device_app_usage_history))
}

/// Create app usage router for organization-level endpoints.
///
/// Routes:
/// - GET /api/admin/v1/organizations/:org_id/app-usage/analytics - Get org-wide analytics
pub fn org_router() -> Router<AppState> {
    Router::new().route("/analytics", get(get_org_app_usage_analytics))
}

/// Get app usage summary for a device.
///
/// GET /api/admin/v1/organizations/:org_id/devices/:device_id/app-usage
#[axum::debug_handler]
async fn get_device_app_usage_summary(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<AppUsageSummaryQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user has access to organization
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (any org member can view app usage)
    if org_user.role != OrgUserRole::Owner
        && org_user.role != OrgUserRole::Admin
        && org_user.role != OrgUserRole::Member
    {
        return Err(ApiError::Forbidden("Insufficient permissions".to_string()));
    }

    // Verify device belongs to organization
    let device_repo = DeviceRepository::new(state.pool.clone());
    let device = device_repo
        .find_fleet_device(org_id, device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Calculate date range
    let today = Utc::now().date_naive();
    let to = query.to.unwrap_or(today);
    let from = query
        .from
        .unwrap_or_else(|| today.checked_sub_days(Days::new(7)).unwrap_or(today));
    let top_apps_limit = query.top_apps_limit.unwrap_or(10).clamp(1, 50);

    // Get usage data
    let app_usage_repo = AppUsageRepository::new(state.pool.clone());
    let summary = app_usage_repo
        .get_device_summary(org_id, device_id, from, to)
        .await?;
    let top_apps = app_usage_repo
        .get_top_apps(org_id, device_id, from, to, top_apps_limit)
        .await?;

    let response = AppUsageSummaryResponse {
        device_id,
        device_name: Some(device.display_name),
        total_foreground_time_ms: summary.total_foreground_time_ms,
        total_background_time_ms: summary.total_background_time_ms,
        total_launches: summary.total_launches as i32,
        unique_apps: summary.unique_apps as i32,
        top_apps: top_apps
            .into_iter()
            .map(|app| AppUsageItem {
                package_name: app.package_name,
                app_name: app.app_name,
                category: app.category,
                foreground_time_ms: app.foreground_time_ms,
                background_time_ms: app.background_time_ms,
                launch_count: app.launch_count as i32,
                notification_count: app.notification_count as i32,
            })
            .collect(),
        period: AppUsagePeriod {
            start: from,
            end: to,
        },
    };

    info!(
        org_id = %org_id,
        device_id = %device_id,
        from = %from,
        to = %to,
        "Retrieved device app usage summary"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Get app usage history for a device.
///
/// GET /api/admin/v1/organizations/:org_id/devices/:device_id/app-usage/history
#[axum::debug_handler]
async fn get_device_app_usage_history(
    State(state): State<AppState>,
    Path((org_id, device_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<AppUsageHistoryQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user has access to organization
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission
    if org_user.role != OrgUserRole::Owner
        && org_user.role != OrgUserRole::Admin
        && org_user.role != OrgUserRole::Member
    {
        return Err(ApiError::Forbidden("Insufficient permissions".to_string()));
    }

    // Verify device belongs to organization
    let device_repo = DeviceRepository::new(state.pool.clone());
    let _device = device_repo
        .find_fleet_device(org_id, device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Device not found".to_string()))?;

    // Pagination
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 100);
    let limit = per_page as i64;
    let offset = ((page - 1) * per_page) as i64;

    // Get history data
    let app_usage_repo = AppUsageRepository::new(state.pool.clone());
    let (records, total) = app_usage_repo
        .get_device_history(
            org_id,
            device_id,
            query.from,
            query.to,
            query.package_name.as_deref(),
            query.category.as_deref(),
            limit,
            offset,
        )
        .await?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    let response = AppUsageHistoryResponse {
        device_id,
        history: records
            .into_iter()
            .map(|r| AppUsageHistoryEntry {
                date: r.usage_date,
                package_name: r.package_name,
                app_name: r.app_name,
                category: r.category,
                foreground_time_ms: r.foreground_time_ms,
                background_time_ms: r.background_time_ms,
                launch_count: r.launch_count,
                notification_count: r.notification_count,
            })
            .collect(),
        pagination: AppUsagePagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    info!(
        org_id = %org_id,
        device_id = %device_id,
        page = page,
        total = total,
        "Retrieved device app usage history"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Get organization-wide app usage analytics.
///
/// GET /api/admin/v1/organizations/:org_id/app-usage/analytics
#[axum::debug_handler]
async fn get_org_app_usage_analytics(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<AppUsageAnalyticsQuery>,
    user: UserAuth,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user has access to organization
    let org_user_repo = OrgUserRepository::new(state.pool.clone());
    let org_user = org_user_repo
        .find_by_org_and_user(org_id, user.user_id)
        .await?
        .ok_or_else(|| ApiError::Forbidden("User not in organization".to_string()))?;

    // Check permission (admin or owner only for org-wide analytics)
    if org_user.role != OrgUserRole::Owner && org_user.role != OrgUserRole::Admin {
        return Err(ApiError::Forbidden(
            "Admin or owner access required".to_string(),
        ));
    }

    // Calculate date range
    let today = Utc::now().date_naive();
    let to = query.to.unwrap_or(today);
    let from = query
        .from
        .unwrap_or_else(|| today.checked_sub_days(Days::new(30)).unwrap_or(today));

    let app_usage_repo = AppUsageRepository::new(state.pool.clone());

    // Get summary
    let summary_entity = app_usage_repo
        .get_org_analytics_summary(org_id, from, to)
        .await?;

    let avg_foreground = if summary_entity.total_devices > 0 {
        summary_entity.total_foreground_time_ms / summary_entity.total_devices
    } else {
        0
    };

    // Get trends
    let trends_entity = app_usage_repo
        .get_org_daily_trends(org_id, from, to)
        .await?;

    // Get top apps
    let top_apps_entity = app_usage_repo
        .get_org_top_apps(org_id, from, to, 10)
        .await?;

    // Get device counts for top apps
    let package_names: Vec<String> = top_apps_entity
        .iter()
        .map(|a| a.package_name.clone())
        .collect();
    let device_counts = app_usage_repo
        .get_app_device_counts(org_id, from, to, &package_names)
        .await?;
    let device_counts_map: std::collections::HashMap<String, i64> =
        device_counts.into_iter().collect();

    // Calculate total foreground time for percentage
    let total_foreground: i64 = top_apps_entity.iter().map(|a| a.foreground_time_ms).sum();

    // Get category usage
    let categories_entity = app_usage_repo
        .get_org_category_usage(org_id, from, to)
        .await?;
    let total_category_time: i64 = categories_entity.iter().map(|c| c.foreground_time_ms).sum();

    let response = AppUsageAnalyticsResponse {
        organization_id: org_id,
        period: AppUsagePeriod {
            start: from,
            end: to,
        },
        summary: AnalyticsSummary {
            total_devices: summary_entity.total_devices as i32,
            total_foreground_time_ms: summary_entity.total_foreground_time_ms,
            total_background_time_ms: summary_entity.total_background_time_ms,
            total_launches: summary_entity.total_launches,
            avg_foreground_time_per_device_ms: avg_foreground,
            unique_apps: summary_entity.unique_apps as i32,
        },
        trends: trends_entity
            .into_iter()
            .map(|t| AnalyticsTrendPoint {
                date: t.date,
                active_devices: t.active_devices as i32,
                foreground_time_ms: t.foreground_time_ms,
                background_time_ms: t.background_time_ms,
                launches: t.launches,
            })
            .collect(),
        top_apps: top_apps_entity
            .into_iter()
            .map(|a| {
                let percentage = if total_foreground > 0 {
                    (a.foreground_time_ms as f64 / total_foreground as f64) * 100.0
                } else {
                    0.0
                };
                TopAppItem {
                    package_name: a.package_name.clone(),
                    app_name: a.app_name,
                    category: a.category,
                    total_foreground_time_ms: a.foreground_time_ms,
                    device_count: *device_counts_map.get(&a.package_name).unwrap_or(&0) as i32,
                    total_launches: a.launch_count,
                    percentage,
                }
            })
            .collect(),
        by_category: categories_entity
            .into_iter()
            .map(|c| {
                let percentage = if total_category_time > 0 {
                    (c.foreground_time_ms as f64 / total_category_time as f64) * 100.0
                } else {
                    0.0
                };
                CategoryUsageItem {
                    category: c.category.unwrap_or_else(|| "Unknown".to_string()),
                    foreground_time_ms: c.foreground_time_ms,
                    app_count: c.app_count as i32,
                    percentage,
                }
            })
            .collect(),
    };

    info!(
        org_id = %org_id,
        from = %from,
        to = %to,
        total_devices = summary_entity.total_devices,
        "Retrieved organization app usage analytics"
    );

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_router_creation() {
        let _router: Router<AppState> = device_router();
    }

    #[test]
    fn test_org_router_creation() {
        let _router: Router<AppState> = org_router();
    }
}
