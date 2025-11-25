//! Location endpoint handlers.

use axum::{extract::State, Json};

use crate::app::AppState;
use crate::error::ApiError;
use domain::models::location::{BatchUploadRequest, UploadLocationRequest, UploadLocationResponse};

/// Upload a single location.
///
/// POST /api/locations
pub async fn upload_location(
    State(_state): State<AppState>,
    Json(_request): Json<UploadLocationRequest>,
) -> Result<Json<UploadLocationResponse>, ApiError> {
    // Implementation will be completed in Story 3.1
    Err(ApiError::Internal("Not implemented yet".to_string()))
}

/// Upload multiple locations in a batch.
///
/// POST /api/locations/batch
pub async fn upload_batch(
    State(_state): State<AppState>,
    Json(_request): Json<BatchUploadRequest>,
) -> Result<Json<UploadLocationResponse>, ApiError> {
    // Implementation will be completed in Story 3.2
    Err(ApiError::Internal("Not implemented yet".to_string()))
}
