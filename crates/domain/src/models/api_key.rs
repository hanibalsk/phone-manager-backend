//! API key domain models for organization-scoped key management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request to create a new organization API key.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateApiKeyRequest {
    /// Human-readable key name (1-100 chars)
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,

    /// Detailed description (max 255 chars)
    #[validate(length(max = 255, message = "Description must be at most 255 characters"))]
    pub description: Option<String>,

    /// Days until expiration (1-365, null = never expires)
    #[validate(range(min = 1, max = 365, message = "Expiration must be 1-365 days"))]
    pub expires_in_days: Option<i32>,
}

/// Request to update an existing API key's metadata.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateApiKeyRequest {
    /// New key name (1-100 chars)
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,

    /// New description (max 255 chars)
    #[validate(length(max = 255, message = "Description must be at most 255 characters"))]
    pub description: Option<String>,
}

/// Response for a single API key (without the actual key value).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiKeyResponse {
    /// Unique key identifier
    pub id: i64,

    /// Key prefix for identification (e.g., "pm_live_ab")
    pub key_prefix: String,

    /// Human-readable key name
    pub name: String,

    /// Detailed description
    pub description: Option<String>,

    /// Whether the key is active
    pub is_active: bool,

    /// Last time the key was used
    pub last_used_at: Option<DateTime<Utc>>,

    /// When the key was created
    pub created_at: DateTime<Utc>,

    /// When the key expires (null = never)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response when creating a new API key (includes the full key, shown only once).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateApiKeyResponse {
    /// Unique key identifier
    pub id: i64,

    /// The full API key (shown ONCE, store securely)
    pub key: String,

    /// Key prefix for identification
    pub key_prefix: String,

    /// Human-readable key name
    pub name: String,

    /// Detailed description
    pub description: Option<String>,

    /// When the key expires (null = never)
    pub expires_at: Option<DateTime<Utc>>,

    /// When the key was created
    pub created_at: DateTime<Utc>,
}

/// Response for listing API keys.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ListApiKeysResponse {
    /// List of API keys
    pub api_keys: Vec<ApiKeyResponse>,

    /// Pagination information
    pub pagination: ApiKeyPagination,
}

/// Pagination information for API key lists.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ApiKeyPagination {
    /// Current page number
    pub page: i32,

    /// Items per page
    pub per_page: i32,

    /// Total number of items
    pub total: i64,

    /// Total number of pages
    pub total_pages: i32,
}

/// Query parameters for listing API keys.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListApiKeysQuery {
    /// Include revoked (inactive) keys
    #[serde(default)]
    pub include_inactive: bool,

    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: i32,

    /// Items per page (max 100)
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_page() -> i32 {
    1
}

fn default_per_page() -> i32 {
    50
}

impl ListApiKeysQuery {
    /// Returns the per_page value, clamped to a maximum of 100.
    pub fn per_page_clamped(&self) -> i32 {
        self.per_page.clamp(1, 100)
    }

    /// Returns the page value, ensuring it's at least 1.
    pub fn page_clamped(&self) -> i32 {
        self.page.max(1)
    }
}

/// Maximum number of API keys allowed per organization.
pub const MAX_API_KEYS_PER_ORG: i64 = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_request_deserialization() {
        let json = r#"{"name": "Production Key", "description": "For mobile app", "expires_in_days": 365}"#;
        let request: CreateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Production Key");
        assert_eq!(request.description, Some("For mobile app".to_string()));
        assert_eq!(request.expires_in_days, Some(365));
    }

    #[test]
    fn test_create_request_minimal() {
        let json = r#"{"name": "Minimal Key"}"#;
        let request: CreateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Minimal Key");
        assert!(request.description.is_none());
        assert!(request.expires_in_days.is_none());
    }

    #[test]
    fn test_create_request_validation_name_too_short() {
        let request = CreateApiKeyRequest {
            name: "".to_string(),
            description: None,
            expires_in_days: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_request_validation_name_too_long() {
        let request = CreateApiKeyRequest {
            name: "a".repeat(101),
            description: None,
            expires_in_days: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_request_validation_expires_too_high() {
        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            description: None,
            expires_in_days: Some(400),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_request_validation_valid() {
        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            description: Some("A description".to_string()),
            expires_in_days: Some(30),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_request_deserialization() {
        let json = r#"{"name": "Updated Name", "description": "Updated description"}"#;
        let request: UpdateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Updated Name".to_string()));
        assert_eq!(request.description, Some("Updated description".to_string()));
    }

    #[test]
    fn test_update_request_partial() {
        let json = r#"{"name": "Only Name"}"#;
        let request: UpdateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, Some("Only Name".to_string()));
        assert!(request.description.is_none());
    }

    #[test]
    fn test_list_query_defaults() {
        let json = r#"{}"#;
        let query: ListApiKeysQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_inactive);
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 50);
    }

    #[test]
    fn test_list_query_per_page_clamped() {
        let query = ListApiKeysQuery {
            include_inactive: false,
            page: 1,
            per_page: 200,
        };
        assert_eq!(query.per_page_clamped(), 100);
    }

    #[test]
    fn test_api_key_response_serialization() {
        let response = ApiKeyResponse {
            id: 42,
            key_prefix: "pm_live_ab".to_string(),
            name: "Test Key".to_string(),
            description: Some("A test key".to_string()),
            is_active: true,
            last_used_at: None,
            created_at: Utc::now(),
            expires_at: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"key_prefix\":\"pm_live_ab\""));
        assert!(json.contains("\"is_active\":true"));
    }

    #[test]
    fn test_create_api_key_response_serialization() {
        let response = CreateApiKeyResponse {
            id: 42,
            key: "pm_live_abc123def456".to_string(),
            key_prefix: "pm_live_ab".to_string(),
            name: "Test Key".to_string(),
            description: None,
            expires_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"key\":\"pm_live_abc123def456\""));
    }
}
