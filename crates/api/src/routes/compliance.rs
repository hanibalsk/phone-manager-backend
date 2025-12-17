//! Compliance route handlers.
//!
//! AP-11.7-8: Compliance Dashboard and Report endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{Duration, Utc};
use tracing::info;
use uuid::Uuid;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::{
    AuditActivitySummary, AuditLogStats, ComplianceAssessment, ComplianceDashboardResponse,
    ComplianceFinding, ComplianceReportFormat, ComplianceReportQuery, ComplianceReportResponse,
    ComplianceStatus, DataRetentionStatus, DataSubjectRequestReportSummary,
    DataSubjectRequestStats, FindingSeverity, OrganizationReportSummary, RequestStatusCounts,
    RequestTypeCount,
};
use persistence::repositories::{
    AuditLogRepository, DashboardRepository, DataSubjectRequestRepository,
};

/// Create compliance router.
///
/// Routes:
/// - GET /api/admin/v1/organizations/:org_id/compliance - Dashboard
/// - GET /api/admin/v1/organizations/:org_id/compliance/report - Generate report
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_compliance_dashboard))
        .route("/report", get(generate_compliance_report))
}

/// Get compliance dashboard.
///
/// GET /api/admin/v1/organizations/:org_id/compliance
#[axum::debug_handler(state = AppState)]
async fn get_compliance_dashboard(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let dsr_repo = DataSubjectRequestRepository::new(state.pool.clone());
    let audit_repo = AuditLogRepository::new(state.pool.clone());

    // Get DSR statistics
    let dsr_counts = dsr_repo.get_counts(org_id).await?;
    let avg_processing_time = dsr_repo
        .get_average_processing_time(org_id, None, None)
        .await?;
    let compliance_rate = dsr_repo.get_compliance_rate(org_id, None, None).await?;

    // Get audit log statistics
    let total_audit_entries = audit_repo.count(org_id).await?;

    // Calculate time-based audit counts
    let now = Utc::now();
    let last_24h = now - Duration::hours(24);
    let last_7d = now - Duration::days(7);
    let last_30d = now - Duration::days(30);

    // For audit log time-based counts, we'll use the list method with date filters
    let query_24h = domain::models::ListAuditLogsQuery {
        from: Some(last_24h),
        ..Default::default()
    };
    let (_, count_24h) = audit_repo.list(org_id, &query_24h).await?;

    let query_7d = domain::models::ListAuditLogsQuery {
        from: Some(last_7d),
        ..Default::default()
    };
    let (_, count_7d) = audit_repo.list(org_id, &query_7d).await?;

    let query_30d = domain::models::ListAuditLogsQuery {
        from: Some(last_30d),
        ..Default::default()
    };
    let (_, count_30d) = audit_repo.list(org_id, &query_30d).await?;

    // Calculate compliance score
    let (score, status, _findings) =
        calculate_compliance_score(&dsr_counts, compliance_rate, avg_processing_time);

    let response = ComplianceDashboardResponse {
        data_subject_requests: DataSubjectRequestStats {
            total: dsr_counts.total,
            pending: dsr_counts.pending,
            in_progress: dsr_counts.in_progress,
            completed: dsr_counts.completed,
            rejected: dsr_counts.rejected,
            overdue: dsr_counts.overdue,
            average_processing_days: avg_processing_time,
            compliance_rate,
        },
        audit_logs: AuditLogStats {
            total_entries: total_audit_entries,
            last_24_hours: count_24h,
            last_7_days: count_7d,
            last_30_days: count_30d,
        },
        data_retention: DataRetentionStatus {
            location_retention_days: state.config.limits.location_retention_days,
            audit_log_retention_days: None, // Could be configured
            cleanup_enabled: true,          // Assume cleanup is enabled
        },
        compliance_score: score,
        status,
    };

    info!(
        org_id = %org_id,
        compliance_score = score,
        "Retrieved compliance dashboard"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Generate compliance report.
///
/// GET /api/admin/v1/organizations/:org_id/compliance/report
#[axum::debug_handler(state = AppState)]
async fn generate_compliance_report(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ComplianceReportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let now = Utc::now();
    let period_end = query.to.unwrap_or(now);
    let period_start = query.from.unwrap_or(period_end - Duration::days(30));

    // Validate format (only JSON supported currently)
    if query.format == Some(ComplianceReportFormat::Pdf) {
        return Err(ApiError::Validation(
            "PDF format is not yet supported. Please use JSON.".to_string(),
        ));
    }

    let dsr_repo = DataSubjectRequestRepository::new(state.pool.clone());
    let audit_repo = AuditLogRepository::new(state.pool.clone());
    let dashboard_repo = DashboardRepository::new(state.pool.clone());

    // Get organization summary
    let dashboard_metrics = dashboard_repo.get_metrics(org_id).await?;

    // Get DSR statistics for the period
    let dsr_counts = dsr_repo.get_counts(org_id).await?;
    let dsr_by_type = dsr_repo
        .get_counts_by_type(org_id, Some(period_start), Some(period_end))
        .await?;
    let avg_processing_time = dsr_repo
        .get_average_processing_time(org_id, Some(period_start), Some(period_end))
        .await?;
    let compliance_rate = dsr_repo
        .get_compliance_rate(org_id, Some(period_start), Some(period_end))
        .await?;

    // Get audit log count for period
    let audit_query = domain::models::ListAuditLogsQuery {
        from: Some(period_start),
        to: Some(period_end),
        ..Default::default()
    };
    let (_, total_audit_entries) = audit_repo.list(org_id, &audit_query).await?;

    // Calculate compliance assessment
    let (score, status, findings) =
        calculate_compliance_score(&dsr_counts, compliance_rate, avg_processing_time);

    let recommendations = generate_recommendations(&findings, &dsr_counts);

    let response = ComplianceReportResponse {
        generated_at: now,
        period_start,
        period_end,
        organization: OrganizationReportSummary {
            total_devices: dashboard_metrics.devices.total,
            total_users: dashboard_metrics.users.total,
            total_groups: dashboard_metrics.groups.total,
        },
        data_subject_requests: DataSubjectRequestReportSummary {
            total_requests: dsr_counts.total,
            by_type: dsr_by_type
                .into_iter()
                .map(|(t, count)| RequestTypeCount {
                    request_type: t.to_string(),
                    count,
                })
                .collect(),
            by_status: RequestStatusCounts {
                pending: dsr_counts.pending,
                in_progress: dsr_counts.in_progress,
                completed: dsr_counts.completed,
                rejected: dsr_counts.rejected,
                cancelled: 0, // Would need separate count
            },
            average_processing_days: avg_processing_time,
            compliance_rate,
        },
        audit_activity: AuditActivitySummary {
            total_entries: total_audit_entries,
            top_actions: vec![], // Would need additional query for top actions
        },
        compliance_assessment: ComplianceAssessment {
            score,
            status,
            findings,
            recommendations,
        },
    };

    info!(
        org_id = %org_id,
        period_start = %period_start,
        period_end = %period_end,
        compliance_score = score,
        "Generated compliance report"
    );

    Ok((StatusCode::OK, Json(response)))
}

/// Calculate compliance score based on DSR metrics.
fn calculate_compliance_score(
    dsr_counts: &persistence::repositories::DataSubjectRequestCounts,
    compliance_rate: Option<f64>,
    avg_processing_time: Option<f64>,
) -> (u8, ComplianceStatus, Vec<ComplianceFinding>) {
    let mut score: f64 = 100.0;
    let mut findings = Vec::new();

    // Deduct points for overdue requests
    if dsr_counts.overdue > 0 {
        let overdue_penalty = (dsr_counts.overdue as f64 * 10.0).min(30.0);
        score -= overdue_penalty;
        findings.push(ComplianceFinding {
            category: "Data Subject Requests".to_string(),
            severity: FindingSeverity::Critical,
            description: format!(
                "{} data subject request(s) are overdue and require immediate attention",
                dsr_counts.overdue
            ),
        });
    }

    // Deduct points for low compliance rate
    if let Some(rate) = compliance_rate {
        if rate < 90.0 {
            let rate_penalty = ((90.0 - rate) / 2.0).min(20.0);
            score -= rate_penalty;
            findings.push(ComplianceFinding {
                category: "Data Subject Requests".to_string(),
                severity: if rate < 70.0 {
                    FindingSeverity::Critical
                } else {
                    FindingSeverity::Warning
                },
                description: format!(
                    "Compliance rate is {:.1}%, below the recommended 90% threshold",
                    rate
                ),
            });
        }
    }

    // Deduct points for slow processing
    if let Some(avg_days) = avg_processing_time {
        if avg_days > 20.0 {
            let time_penalty = ((avg_days - 20.0) / 2.0).min(15.0);
            score -= time_penalty;
            findings.push(ComplianceFinding {
                category: "Data Subject Requests".to_string(),
                severity: if avg_days > 25.0 {
                    FindingSeverity::Warning
                } else {
                    FindingSeverity::Info
                },
                description: format!(
                    "Average processing time is {:.1} days, approaching the 30-day GDPR limit",
                    avg_days
                ),
            });
        }
    }

    // Add informational finding for pending requests
    if dsr_counts.pending > 0 {
        findings.push(ComplianceFinding {
            category: "Data Subject Requests".to_string(),
            severity: FindingSeverity::Info,
            description: format!(
                "{} data subject request(s) are pending review",
                dsr_counts.pending
            ),
        });
    }

    // Determine status based on score
    let score_u8 = score.clamp(0.0, 100.0) as u8;
    let status = if score_u8 >= 90 {
        ComplianceStatus::Compliant
    } else if score_u8 >= 70 {
        ComplianceStatus::NeedsAttention
    } else {
        ComplianceStatus::NonCompliant
    };

    (score_u8, status, findings)
}

/// Generate recommendations based on findings.
fn generate_recommendations(
    findings: &[ComplianceFinding],
    dsr_counts: &persistence::repositories::DataSubjectRequestCounts,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Check for critical findings
    let has_overdue = findings
        .iter()
        .any(|f| f.severity == FindingSeverity::Critical && f.description.contains("overdue"));

    if has_overdue {
        recommendations.push(
            "Prioritize processing overdue data subject requests to avoid regulatory penalties"
                .to_string(),
        );
    }

    // Check for low compliance rate
    let has_low_rate = findings
        .iter()
        .any(|f| f.description.contains("Compliance rate"));

    if has_low_rate {
        recommendations.push(
            "Implement automated workflows to ensure timely processing of data subject requests"
                .to_string(),
        );
    }

    // Check for slow processing
    let has_slow_processing = findings
        .iter()
        .any(|f| f.description.contains("Average processing time"));

    if has_slow_processing {
        recommendations.push(
            "Review and optimize data subject request handling procedures to reduce processing time"
                .to_string(),
        );
    }

    // General recommendation if there are pending requests
    if dsr_counts.pending > 5 {
        recommendations.push(
            "Consider assigning additional resources to handle the backlog of pending requests"
                .to_string(),
        );
    }

    // Add general best practice if no specific issues
    if recommendations.is_empty() {
        recommendations.push(
            "Continue monitoring compliance metrics and maintain current processing standards"
                .to_string(),
        );
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use persistence::repositories::DataSubjectRequestCounts;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }

    #[test]
    fn test_compliance_score_perfect() {
        let counts = DataSubjectRequestCounts {
            total: 10,
            pending: 0,
            in_progress: 0,
            completed: 10,
            rejected: 0,
            overdue: 0,
        };

        let (score, status, findings) =
            calculate_compliance_score(&counts, Some(100.0), Some(10.0));

        assert_eq!(score, 100);
        assert_eq!(status, ComplianceStatus::Compliant);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_compliance_score_with_overdue() {
        let counts = DataSubjectRequestCounts {
            total: 10,
            pending: 2,
            in_progress: 1,
            completed: 5,
            rejected: 0,
            overdue: 2,
        };

        let (score, _status, findings) =
            calculate_compliance_score(&counts, Some(95.0), Some(15.0));

        assert!(score < 100);
        assert!(findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Critical));
    }

    #[test]
    fn test_compliance_score_needs_attention() {
        let counts = DataSubjectRequestCounts {
            total: 10,
            pending: 3,
            in_progress: 2,
            completed: 5,
            rejected: 0,
            overdue: 1, // Add overdue to trigger deductions
        };

        // With overdue=1 (10 point deduction), compliance_rate=60% (15 point deduction),
        // and slow processing (5 point deduction), we get ~70
        let (score, status, _) = calculate_compliance_score(&counts, Some(60.0), Some(25.0));

        assert!(score < 90, "Score {} should be < 90", score);
        assert!(score >= 70, "Score {} should be >= 70", score);
        assert_eq!(status, ComplianceStatus::NeedsAttention);
    }

    #[test]
    fn test_generate_recommendations_with_overdue() {
        let findings = vec![ComplianceFinding {
            category: "Data Subject Requests".to_string(),
            severity: FindingSeverity::Critical,
            description: "2 data subject request(s) are overdue".to_string(),
        }];
        let counts = DataSubjectRequestCounts::default();

        let recommendations = generate_recommendations(&findings, &counts);

        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("overdue")));
    }
}
