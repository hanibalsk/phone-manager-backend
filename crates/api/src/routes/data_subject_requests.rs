//! Data Subject Request route handlers.
//!
//! AP-11.4-6: GDPR/CCPA Data Subject Request management endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::{
    CreateDataSubjectRequestRequest, DataSubjectRequestAction, DataSubjectRequestPagination,
    DataSubjectRequestResponse, DataSubjectRequestStatus, DataSubjectRequestType,
    ListDataSubjectRequestsQuery as DomainQuery, ListDataSubjectRequestsResponse,
    ProcessDataSubjectRequestRequest, ProcessorInfo,
};
use persistence::entities::{
    DataSubjectRequestStatusDb, DataSubjectRequestTypeDb, DataSubjectRequestWithProcessorEntity,
};
use persistence::repositories::{
    CreateDataSubjectRequestInput, DataSubjectRequestRepository,
    ListDataSubjectRequestsQuery as RepoQuery, ProcessDataSubjectRequestInput,
};

/// Create data subject requests router.
///
/// Routes:
/// - GET /api/admin/v1/organizations/:org_id/data-requests - List requests
/// - POST /api/admin/v1/organizations/:org_id/data-requests - Create request
/// - GET /api/admin/v1/organizations/:org_id/data-requests/:request_id - Get request
/// - POST /api/admin/v1/organizations/:org_id/data-requests/:request_id/process - Process request
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_data_subject_requests).post(create_data_subject_request))
        .route("/:request_id", get(get_data_subject_request))
        .route("/:request_id/process", post(process_data_subject_request))
}

/// List data subject requests.
///
/// GET /api/admin/v1/organizations/:org_id/data-requests
#[axum::debug_handler(state = AppState)]
async fn list_data_subject_requests(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<DomainQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = DataSubjectRequestRepository::new(state.pool.clone());

    let repo_query = RepoQuery {
        status: query.status.map(status_to_db),
        request_type: query.request_type.map(type_to_db),
        subject_email: query.subject_email.clone(),
        from: query.from,
        to: query.to,
        page: query.page,
        per_page: query.per_page,
    };

    let (entities, total) = repo.list(org_id, &repo_query).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    let requests: Vec<DataSubjectRequestResponse> =
        entities.into_iter().map(entity_to_response).collect();

    let response = ListDataSubjectRequestsResponse {
        requests,
        pagination: DataSubjectRequestPagination {
            page,
            per_page,
            total,
            total_pages,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Create a new data subject request.
///
/// POST /api/admin/v1/organizations/:org_id/data-requests
#[axum::debug_handler(state = AppState)]
async fn create_data_subject_request(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(request): Json<CreateDataSubjectRequestRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Invalid request: {}", e))
    })?;

    let repo = DataSubjectRequestRepository::new(state.pool.clone());

    let input = CreateDataSubjectRequestInput {
        organization_id: org_id,
        request_type: type_to_db(request.request_type),
        subject_email: request.subject_email.clone(),
        subject_name: request.subject_name.clone(),
        subject_user_id: request.subject_user_id,
        description: request.description.clone(),
        due_days: request.due_days,
    };

    let entity = repo.create(input).await?;

    info!(
        org_id = %org_id,
        request_id = %entity.id,
        request_type = %entity.request_type,
        subject_email = %entity.subject_email,
        "Created data subject request"
    );

    // Convert entity to response (without processor info since it's new)
    let response = DataSubjectRequestResponse {
        id: entity.id,
        request_type: type_from_db(entity.request_type),
        status: status_from_db(entity.status),
        subject_email: entity.subject_email,
        subject_name: entity.subject_name,
        subject_user_id: entity.subject_user_id,
        description: entity.description,
        rejection_reason: entity.rejection_reason,
        processor: None,
        result_data: entity.result_data,
        result_file_url: entity.result_file_url,
        result_expires_at: entity.result_expires_at,
        due_date: entity.due_date,
        is_overdue: is_overdue(&entity.status, entity.due_date),
        created_at: entity.created_at,
        updated_at: entity.updated_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a specific data subject request.
///
/// GET /api/admin/v1/organizations/:org_id/data-requests/:request_id
#[axum::debug_handler(state = AppState)]
async fn get_data_subject_request(
    State(state): State<AppState>,
    Path((org_id, request_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let repo = DataSubjectRequestRepository::new(state.pool.clone());

    let entity = repo.find_by_id(org_id, request_id).await?;

    match entity {
        Some(e) => Ok((StatusCode::OK, Json(entity_to_response(e)))),
        None => Err(ApiError::NotFound("Data subject request not found".to_string())),
    }
}

/// Process a data subject request (start, complete, reject, cancel).
///
/// POST /api/admin/v1/organizations/:org_id/data-requests/:request_id/process
#[axum::debug_handler(state = AppState)]
async fn process_data_subject_request(
    State(state): State<AppState>,
    Path((org_id, request_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProcessDataSubjectRequestRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation(format!("Invalid request: {}", e))
    })?;

    // Validate rejection reason is provided when rejecting
    if request.action == DataSubjectRequestAction::Reject && request.rejection_reason.is_none() {
        return Err(ApiError::Validation(
            "Rejection reason is required when rejecting a request".to_string(),
        ));
    }

    let repo = DataSubjectRequestRepository::new(state.pool.clone());

    // Get current request to verify state transition
    let current = repo.find_by_id(org_id, request_id).await?;
    let current = current.ok_or_else(|| {
        ApiError::NotFound("Data subject request not found".to_string())
    })?;

    // Validate state transition
    let new_status = validate_state_transition(
        status_from_db(current.status),
        request.action,
    )?;

    // For now, we'll use a placeholder user ID
    // In a real implementation, this would come from the authenticated admin user
    let processed_by = Uuid::nil(); // Placeholder - should come from auth context

    let input = ProcessDataSubjectRequestInput {
        status: status_to_db(new_status),
        processed_by,
        rejection_reason: request.rejection_reason.clone(),
        result_data: request.result_data.clone(),
        result_file_url: request.result_file_url.clone(),
        result_expires_days: request.result_expires_days,
    };

    let entity = repo.process(org_id, request_id, input).await?;

    let entity = entity.ok_or_else(|| {
        ApiError::NotFound("Data subject request not found".to_string())
    })?;

    info!(
        org_id = %org_id,
        request_id = %entity.id,
        action = ?request.action,
        new_status = %entity.status,
        "Processed data subject request"
    );

    // Convert entity to response
    let response = DataSubjectRequestResponse {
        id: entity.id,
        request_type: type_from_db(entity.request_type),
        status: status_from_db(entity.status),
        subject_email: entity.subject_email,
        subject_name: entity.subject_name,
        subject_user_id: entity.subject_user_id,
        description: entity.description,
        rejection_reason: entity.rejection_reason,
        processor: entity.processed_by.map(|user_id| ProcessorInfo {
            user_id,
            email: None, // Would need to join with users table
            processed_at: entity.processed_at.unwrap_or_else(Utc::now),
        }),
        result_data: entity.result_data,
        result_file_url: entity.result_file_url,
        result_expires_at: entity.result_expires_at,
        due_date: entity.due_date,
        is_overdue: is_overdue(&entity.status, entity.due_date),
        created_at: entity.created_at,
        updated_at: entity.updated_at,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Validate state transition for data subject request.
fn validate_state_transition(
    current: DataSubjectRequestStatus,
    action: DataSubjectRequestAction,
) -> Result<DataSubjectRequestStatus, ApiError> {
    match (current, action) {
        // From pending
        (DataSubjectRequestStatus::Pending, DataSubjectRequestAction::StartProcessing) => {
            Ok(DataSubjectRequestStatus::InProgress)
        }
        (DataSubjectRequestStatus::Pending, DataSubjectRequestAction::Reject) => {
            Ok(DataSubjectRequestStatus::Rejected)
        }
        (DataSubjectRequestStatus::Pending, DataSubjectRequestAction::Cancel) => {
            Ok(DataSubjectRequestStatus::Cancelled)
        }
        // From in_progress
        (DataSubjectRequestStatus::InProgress, DataSubjectRequestAction::Complete) => {
            Ok(DataSubjectRequestStatus::Completed)
        }
        (DataSubjectRequestStatus::InProgress, DataSubjectRequestAction::Reject) => {
            Ok(DataSubjectRequestStatus::Rejected)
        }
        (DataSubjectRequestStatus::InProgress, DataSubjectRequestAction::Cancel) => {
            Ok(DataSubjectRequestStatus::Cancelled)
        }
        // Invalid transitions
        (current, action) => Err(ApiError::Validation(format!(
            "Invalid state transition: cannot {} a {} request",
            format!("{:?}", action).to_lowercase(),
            current
        ))),
    }
}

/// Convert entity to response.
fn entity_to_response(entity: DataSubjectRequestWithProcessorEntity) -> DataSubjectRequestResponse {
    DataSubjectRequestResponse {
        id: entity.id,
        request_type: type_from_db(entity.request_type),
        status: status_from_db(entity.status),
        subject_email: entity.subject_email,
        subject_name: entity.subject_name,
        subject_user_id: entity.subject_user_id,
        description: entity.description,
        rejection_reason: entity.rejection_reason,
        processor: entity.processed_by.map(|user_id| ProcessorInfo {
            user_id,
            email: entity.processor_email.clone(),
            processed_at: entity.processed_at.unwrap_or_else(Utc::now),
        }),
        result_data: entity.result_data,
        result_file_url: entity.result_file_url,
        result_expires_at: entity.result_expires_at,
        due_date: entity.due_date,
        is_overdue: is_overdue(&entity.status, entity.due_date),
        created_at: entity.created_at,
        updated_at: entity.updated_at,
    }
}

/// Check if a request is overdue.
fn is_overdue(status: &DataSubjectRequestStatusDb, due_date: chrono::DateTime<chrono::Utc>) -> bool {
    matches!(
        status,
        DataSubjectRequestStatusDb::Pending | DataSubjectRequestStatusDb::InProgress
    ) && due_date < Utc::now()
}

/// Convert domain request type to database enum.
fn type_to_db(t: DataSubjectRequestType) -> DataSubjectRequestTypeDb {
    match t {
        DataSubjectRequestType::Access => DataSubjectRequestTypeDb::Access,
        DataSubjectRequestType::Deletion => DataSubjectRequestTypeDb::Deletion,
        DataSubjectRequestType::Portability => DataSubjectRequestTypeDb::Portability,
        DataSubjectRequestType::Rectification => DataSubjectRequestTypeDb::Rectification,
        DataSubjectRequestType::Restriction => DataSubjectRequestTypeDb::Restriction,
        DataSubjectRequestType::Objection => DataSubjectRequestTypeDb::Objection,
    }
}

/// Convert database enum to domain request type.
fn type_from_db(t: DataSubjectRequestTypeDb) -> DataSubjectRequestType {
    match t {
        DataSubjectRequestTypeDb::Access => DataSubjectRequestType::Access,
        DataSubjectRequestTypeDb::Deletion => DataSubjectRequestType::Deletion,
        DataSubjectRequestTypeDb::Portability => DataSubjectRequestType::Portability,
        DataSubjectRequestTypeDb::Rectification => DataSubjectRequestType::Rectification,
        DataSubjectRequestTypeDb::Restriction => DataSubjectRequestType::Restriction,
        DataSubjectRequestTypeDb::Objection => DataSubjectRequestType::Objection,
    }
}

/// Convert domain status to database enum.
fn status_to_db(s: DataSubjectRequestStatus) -> DataSubjectRequestStatusDb {
    match s {
        DataSubjectRequestStatus::Pending => DataSubjectRequestStatusDb::Pending,
        DataSubjectRequestStatus::InProgress => DataSubjectRequestStatusDb::InProgress,
        DataSubjectRequestStatus::Completed => DataSubjectRequestStatusDb::Completed,
        DataSubjectRequestStatus::Rejected => DataSubjectRequestStatusDb::Rejected,
        DataSubjectRequestStatus::Cancelled => DataSubjectRequestStatusDb::Cancelled,
    }
}

/// Convert database enum to domain status.
fn status_from_db(s: DataSubjectRequestStatusDb) -> DataSubjectRequestStatus {
    match s {
        DataSubjectRequestStatusDb::Pending => DataSubjectRequestStatus::Pending,
        DataSubjectRequestStatusDb::InProgress => DataSubjectRequestStatus::InProgress,
        DataSubjectRequestStatusDb::Completed => DataSubjectRequestStatus::Completed,
        DataSubjectRequestStatusDb::Rejected => DataSubjectRequestStatus::Rejected,
        DataSubjectRequestStatusDb::Cancelled => DataSubjectRequestStatus::Cancelled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }

    #[test]
    fn test_validate_state_transition_pending_to_in_progress() {
        let result = validate_state_transition(
            DataSubjectRequestStatus::Pending,
            DataSubjectRequestAction::StartProcessing,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataSubjectRequestStatus::InProgress);
    }

    #[test]
    fn test_validate_state_transition_pending_to_rejected() {
        let result = validate_state_transition(
            DataSubjectRequestStatus::Pending,
            DataSubjectRequestAction::Reject,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataSubjectRequestStatus::Rejected);
    }

    #[test]
    fn test_validate_state_transition_in_progress_to_completed() {
        let result = validate_state_transition(
            DataSubjectRequestStatus::InProgress,
            DataSubjectRequestAction::Complete,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataSubjectRequestStatus::Completed);
    }

    #[test]
    fn test_validate_state_transition_invalid() {
        let result = validate_state_transition(
            DataSubjectRequestStatus::Completed,
            DataSubjectRequestAction::StartProcessing,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_type_conversion_roundtrip() {
        let original = DataSubjectRequestType::Access;
        let db = type_to_db(original);
        let back = type_from_db(db);
        assert_eq!(original, back);
    }

    #[test]
    fn test_status_conversion_roundtrip() {
        let original = DataSubjectRequestStatus::InProgress;
        let db = status_to_db(original);
        let back = status_from_db(db);
        assert_eq!(original, back);
    }
}
