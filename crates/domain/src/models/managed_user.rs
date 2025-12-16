//! Managed user models for admin user management (Epic 9).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Managed user information for admin listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ManagedUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub tracking_enabled: bool,
    pub device_count: i64,
    pub last_location: Option<UserLastLocation>,
    pub organization_id: Option<Uuid>,
    pub organization_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// User's last known location (from any of their devices).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UserLastLocation {
    pub device_id: Uuid,
    pub device_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32,
    pub captured_at: DateTime<Utc>,
}

/// Query parameters for listing managed users.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ListManagedUsersQuery {
    #[validate(length(max = 100, message = "Search query must be 100 characters or less"))]
    pub search: Option<String>,
    pub tracking_enabled: Option<bool>,
    #[validate(range(min = 1, max = 100, message = "Page must be between 1 and 100"))]
    #[serde(default = "default_page")]
    pub page: u32,
    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    50
}

/// Response for listing managed users.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListManagedUsersResponse {
    pub users: Vec<ManagedUser>,
    pub pagination: ManagedUserPagination,
}

/// Pagination info for managed users.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ManagedUserPagination {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

/// Request to update user tracking status.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateTrackingRequest {
    pub enabled: bool,
}

/// Response for tracking update.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateTrackingResponse {
    pub user_id: Uuid,
    pub tracking_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

/// Response for removing managed user.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RemoveManagedUserResponse {
    pub user_id: Uuid,
    pub removed: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_managed_users_query_defaults() {
        let json = r#"{}"#;
        let query: ListManagedUsersQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 50);
        assert!(query.search.is_none());
        assert!(query.tracking_enabled.is_none());
    }

    #[test]
    fn test_list_managed_users_query_with_values() {
        let json = r#"{"search": "test", "tracking_enabled": true, "page": 2, "per_page": 25}"#;
        let query: ListManagedUsersQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.search, Some("test".to_string()));
        assert_eq!(query.tracking_enabled, Some(true));
        assert_eq!(query.page, 2);
        assert_eq!(query.per_page, 25);
    }

    #[test]
    fn test_update_tracking_request() {
        let json = r#"{"enabled": false}"#;
        let request: UpdateTrackingRequest = serde_json::from_str(json).unwrap();
        assert!(!request.enabled);
    }
}
