use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)] // Variants used in future stories
pub enum ApiError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<ValidationDetail>>,
}

#[derive(Debug, Serialize)]
pub struct ValidationDetail {
    pub field: String,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg.clone()),
            ApiError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Too many requests. Please try again later.".into(),
            ),
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred".into(),
                )
            }
            ApiError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                msg.clone(),
            ),
        };

        let body = ErrorBody {
            error: error_code.into(),
            message,
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".into()),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => ApiError::Conflict("Resource already exists".into()),
                        "23503" => ApiError::NotFound("Referenced resource not found".into()),
                        _ => ApiError::Internal(format!("Database error: {}", db_err)),
                    }
                } else {
                    ApiError::Internal(format!("Database error: {}", db_err))
                }
            }
            _ => ApiError::Internal(format!("Database error: {}", err)),
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let details: Vec<ValidationDetail> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| ValidationDetail {
                    field: field.to_string(),
                    message: e.message.clone().map(|m| m.to_string()).unwrap_or_default(),
                })
            })
            .collect();

        let message = if details.len() == 1 {
            details[0].message.clone()
        } else {
            format!("{} validation errors", details.len())
        };

        ApiError::Validation(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_api_error_unauthorized() {
        let error = ApiError::Unauthorized("test message".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_api_error_forbidden() {
        let error = ApiError::Forbidden("access denied".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_api_error_not_found() {
        let error = ApiError::NotFound("resource not found".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_api_error_conflict() {
        let error = ApiError::Conflict("already exists".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_api_error_validation() {
        let error = ApiError::Validation("invalid input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_rate_limited() {
        let error = ApiError::RateLimited;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_api_error_internal() {
        let error = ApiError::Internal("database connection failed".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_api_error_service_unavailable() {
        let error = ApiError::ServiceUnavailable("maintenance".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_api_error_display() {
        assert_eq!(
            format!("{}", ApiError::Unauthorized("test".to_string())),
            "Unauthorized: test"
        );
        assert_eq!(
            format!("{}", ApiError::Forbidden("test".to_string())),
            "Forbidden: test"
        );
        assert_eq!(
            format!("{}", ApiError::NotFound("test".to_string())),
            "Not found: test"
        );
        assert_eq!(
            format!("{}", ApiError::Conflict("test".to_string())),
            "Conflict: test"
        );
        assert_eq!(
            format!("{}", ApiError::Validation("test".to_string())),
            "Validation error: test"
        );
        assert_eq!(format!("{}", ApiError::RateLimited), "Rate limited");
        assert_eq!(
            format!("{}", ApiError::Internal("test".to_string())),
            "Internal error: test"
        );
        assert_eq!(
            format!("{}", ApiError::ServiceUnavailable("test".to_string())),
            "Service unavailable: test"
        );
    }

    #[test]
    fn test_validation_detail() {
        let detail = ValidationDetail {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
        };
        assert_eq!(detail.field, "email");
        assert_eq!(detail.message, "Invalid email format");
    }

    #[test]
    fn test_from_sqlx_row_not_found() {
        let error: ApiError = sqlx::Error::RowNotFound.into();
        match error {
            ApiError::NotFound(msg) => assert_eq!(msg, "Resource not found"),
            _ => panic!("Expected NotFound error"),
        }
    }
}
