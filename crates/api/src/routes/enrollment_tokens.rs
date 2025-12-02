//! Enrollment token management routes.
//!
//! Story 13.4: Enrollment Tokens Management Endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use persistence::repositories::EnrollmentTokenRepository;
use uuid::Uuid;
use validator::Validate;

use crate::app::AppState;
use crate::error::ApiError;
use crate::extractors::UserAuth;
use domain::models::{
    calculate_expiry, extract_prefix, generate_token, CreateEnrollmentTokenRequest,
    EnrollmentToken, EnrollmentTokenPagination, EnrollmentTokenResponse, ListEnrollmentTokensQuery,
    ListEnrollmentTokensResponse, QrCodeResponse,
};

/// Create a new enrollment token.
///
/// POST /api/admin/v1/organizations/:org_id/enrollment-tokens
///
/// Requires JWT authentication.
pub async fn create_enrollment_token(
    State(state): State<AppState>,
    user_auth: UserAuth,
    Path(org_id): Path<Uuid>,
    Json(request): Json<CreateEnrollmentTokenRequest>,
) -> Result<(StatusCode, Json<EnrollmentTokenResponse>), ApiError> {
    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation error: {}", e)))?;

    let repo = EnrollmentTokenRepository::new(state.pool.clone());

    // Generate token
    let token = generate_token();
    let token_prefix = extract_prefix(&token);

    // Calculate expiry if specified
    let expires_at = request.expires_in_days.map(calculate_expiry);

    // Create token with creator tracking
    let enrollment_token = repo
        .create(
            org_id,
            &token,
            &token_prefix,
            request.group_id.as_deref(),
            request.policy_id,
            request.max_uses,
            expires_at,
            request.auto_assign_user_by_email,
            Some(user_auth.user_id),
        )
        .await?;

    tracing::info!(
        token_id = %enrollment_token.id,
        organization_id = %org_id,
        token_prefix = %token_prefix,
        created_by = %user_auth.user_id,
        "Enrollment token created"
    );

    Ok((StatusCode::CREATED, Json(enrollment_token.into())))
}

/// List enrollment tokens for an organization.
///
/// GET /api/admin/v1/organizations/:org_id/enrollment-tokens
pub async fn list_enrollment_tokens(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListEnrollmentTokensQuery>,
) -> Result<Json<ListEnrollmentTokensResponse>, ApiError> {
    let repo = EnrollmentTokenRepository::new(state.pool.clone());

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    let (tokens, total): (Vec<EnrollmentToken>, i64) = repo.list(org_id, &query).await?;

    let response = ListEnrollmentTokensResponse {
        data: tokens.into_iter().map(Into::into).collect(),
        pagination: EnrollmentTokenPagination {
            page,
            per_page,
            total,
        },
    };

    Ok(Json(response))
}

/// Get a specific enrollment token.
///
/// GET /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id
pub async fn get_enrollment_token(
    State(state): State<AppState>,
    Path((org_id, token_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<EnrollmentTokenResponse>, ApiError> {
    let repo = EnrollmentTokenRepository::new(state.pool.clone());

    let token: EnrollmentToken = repo
        .find_by_id(token_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Enrollment token not found".to_string()))?;

    // Verify token belongs to the organization
    if token.organization_id != org_id {
        return Err(ApiError::NotFound("Enrollment token not found".to_string()));
    }

    Ok(Json(token.into()))
}

/// Revoke an enrollment token.
///
/// DELETE /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id
pub async fn revoke_enrollment_token(
    State(state): State<AppState>,
    Path((org_id, token_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let repo = EnrollmentTokenRepository::new(state.pool.clone());

    // Verify token exists and belongs to organization
    let token: EnrollmentToken = repo
        .find_by_id(token_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Enrollment token not found".to_string()))?;

    if token.organization_id != org_id {
        return Err(ApiError::NotFound("Enrollment token not found".to_string()));
    }

    // Check if already revoked
    if token.is_revoked() {
        return Err(ApiError::Conflict(
            "Enrollment token is already revoked".to_string(),
        ));
    }

    // Revoke token
    repo.revoke(token_id).await?;

    tracing::info!(
        token_id = %token_id,
        organization_id = %org_id,
        "Enrollment token revoked"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// Get QR code for an enrollment token.
///
/// GET /api/admin/v1/organizations/:org_id/enrollment-tokens/:token_id/qr
pub async fn get_enrollment_token_qr(
    State(state): State<AppState>,
    Path((org_id, token_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<QrCodeResponse>, ApiError> {
    let repo = EnrollmentTokenRepository::new(state.pool.clone());

    // Verify token exists and belongs to organization
    let token: EnrollmentToken = repo
        .find_by_id(token_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Enrollment token not found".to_string()))?;

    if token.organization_id != org_id {
        return Err(ApiError::NotFound("Enrollment token not found".to_string()));
    }

    // Check if token is still valid
    if !token.is_valid() {
        return Err(ApiError::Conflict(
            "Cannot generate QR code for invalid token".to_string(),
        ));
    }

    // Build enrollment URL using configured app base URL
    let enrollment_url = format!(
        "{}/enroll?token={}",
        state.config.server.app_base_url.trim_end_matches('/'),
        token.token
    );

    // For now, return just the URL. QR code generation can be added later
    // using a crate like `qrcode` if needed.
    // The mobile app can generate the QR code client-side from the URL.
    let response = QrCodeResponse {
        qr_data: enrollment_url.clone(),
        enrollment_url,
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_enrollment_token_request_validation() {
        let request = CreateEnrollmentTokenRequest {
            group_id: Some("grp123".to_string()),
            policy_id: Some(Uuid::new_v4()),
            max_uses: Some(50),
            expires_in_days: Some(30),
            auto_assign_user_by_email: true,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_enrollment_token_request_minimal() {
        let request = CreateEnrollmentTokenRequest {
            group_id: None,
            policy_id: None,
            max_uses: None,
            expires_in_days: None,
            auto_assign_user_by_email: false,
        };
        assert!(request.validate().is_ok());
    }
}
