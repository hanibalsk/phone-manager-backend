//! Organization domain models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

/// Plan types available for organizations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanType {
    Free,
    Starter,
    Business,
    Enterprise,
}

impl PlanType {
    /// Get the default limits for this plan type.
    pub fn default_limits(&self) -> (i32, i32, i32) {
        // (max_users, max_devices, max_groups)
        match self {
            PlanType::Free => (5, 10, 5),
            PlanType::Starter => (25, 100, 20),
            PlanType::Business => (100, 500, 50),
            PlanType::Enterprise => (i32::MAX, i32::MAX, i32::MAX), // Unlimited
        }
    }
}

impl FromStr for PlanType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(PlanType::Free),
            "starter" => Ok(PlanType::Starter),
            "business" => Ok(PlanType::Business),
            "enterprise" => Ok(PlanType::Enterprise),
            _ => Err(format!("Unknown plan type: {}", s)),
        }
    }
}

impl std::fmt::Display for PlanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanType::Free => write!(f, "free"),
            PlanType::Starter => write!(f, "starter"),
            PlanType::Business => write!(f, "business"),
            PlanType::Enterprise => write!(f, "enterprise"),
        }
    }
}

/// Organization domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub billing_email: String,
    pub plan_type: PlanType,
    pub max_users: i32,
    pub max_devices: i32,
    pub max_groups: i32,
    pub settings: JsonValue,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Organization with usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrganizationWithUsage {
    #[serde(flatten)]
    pub organization: Organization,
    pub current_users: i64,
    pub current_devices: i64,
    pub current_groups: i64,
}

/// Organization usage statistics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrganizationUsageResponse {
    pub organization_id: Uuid,
    pub users: UsageMetric,
    pub devices: DeviceUsageMetric,
    pub groups: UsageMetric,
    pub period: String,
}

/// Generic usage metric with current, max, and percentage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UsageMetric {
    pub current: i64,
    pub max: i32,
    pub percentage: f64,
}

/// Device usage metric with status breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceUsageMetric {
    pub current: i64,
    pub max: i32,
    pub percentage: f64,
    pub by_status: DeviceStatusCounts,
}

/// Device counts by enrollment status.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct DeviceStatusCounts {
    pub enrolled: i64,
    pub pending: i64,
    pub suspended: i64,
    pub retired: i64,
}

/// Request to create a new organization.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CreateOrganizationRequest {
    #[validate(length(min = 2, max = 255, message = "Name must be 2-255 characters"))]
    pub name: String,
    #[validate(length(
        min = 3,
        max = 50,
        message = "Slug must be 3-50 characters"
    ))]
    #[validate(custom(function = "validate_slug"))]
    pub slug: String,
    #[validate(email(message = "Invalid billing email format"))]
    pub billing_email: String,
    pub plan_type: Option<PlanType>,
    #[serde(default)]
    pub settings: Option<JsonValue>,
}

/// Validate slug format: lowercase alphanumeric with hyphens, no leading/trailing hyphens.
fn validate_slug(slug: &str) -> Result<(), validator::ValidationError> {
    if SLUG_REGEX.is_match(slug) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("slug_format")
            .with_message(std::borrow::Cow::Borrowed("Slug must be lowercase alphanumeric with hyphens, no leading/trailing hyphens")))
    }
}

/// Request to update an organization.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct UpdateOrganizationRequest {
    #[validate(length(min = 2, max = 255, message = "Name must be 2-255 characters"))]
    pub name: Option<String>,
    #[validate(email(message = "Invalid billing email format"))]
    pub billing_email: Option<String>,
    pub plan_type: Option<PlanType>,
    pub max_users: Option<i32>,
    pub max_devices: Option<i32>,
    pub max_groups: Option<i32>,
    pub settings: Option<JsonValue>,
}

/// Response for organization creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateOrganizationResponse {
    #[serde(flatten)]
    pub organization: Organization,
}

/// Response for organization list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ListOrganizationsResponse {
    pub data: Vec<Organization>,
    pub pagination: OrganizationPagination,
}

/// Pagination info for organization list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrganizationPagination {
    pub page: i32,
    pub per_page: i32,
    pub total: i64,
    pub total_pages: i32,
}

/// Query parameters for listing organizations.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ListOrganizationsQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub is_active: Option<bool>,
    pub plan_type: Option<PlanType>,
    pub search: Option<String>,
}

// Regex for slug validation
lazy_static::lazy_static! {
    pub static ref SLUG_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_plan_type_serialization() {
        assert_eq!(
            serde_json::to_string(&PlanType::Business).unwrap(),
            "\"business\""
        );
    }

    #[test]
    fn test_plan_type_deserialization() {
        let plan: PlanType = serde_json::from_str("\"enterprise\"").unwrap();
        assert_eq!(plan, PlanType::Enterprise);
    }

    #[test]
    fn test_plan_type_from_str() {
        assert_eq!(PlanType::from_str("free").unwrap(), PlanType::Free);
        assert_eq!(PlanType::from_str("BUSINESS").unwrap(), PlanType::Business);
        assert!(PlanType::from_str("invalid").is_err());
    }

    #[test]
    fn test_plan_type_default_limits() {
        let (users, devices, groups) = PlanType::Free.default_limits();
        assert_eq!(users, 5);
        assert_eq!(devices, 10);
        assert_eq!(groups, 5);

        let (users, devices, groups) = PlanType::Business.default_limits();
        assert_eq!(users, 100);
        assert_eq!(devices, 500);
        assert_eq!(groups, 50);
    }

    #[test]
    fn test_create_organization_request_validation() {
        use validator::Validate;

        // Valid request
        let valid = CreateOrganizationRequest {
            name: "Acme Corp".to_string(),
            slug: "acme-corp".to_string(),
            billing_email: "billing@acme.com".to_string(),
            plan_type: Some(PlanType::Business),
            settings: None,
        };
        assert!(valid.validate().is_ok());

        // Invalid slug (uppercase)
        let invalid_slug = CreateOrganizationRequest {
            name: "Acme Corp".to_string(),
            slug: "Acme-Corp".to_string(),
            billing_email: "billing@acme.com".to_string(),
            plan_type: None,
            settings: None,
        };
        assert!(invalid_slug.validate().is_err());

        // Invalid email
        let invalid_email = CreateOrganizationRequest {
            name: "Acme Corp".to_string(),
            slug: "acme-corp".to_string(),
            billing_email: "not-an-email".to_string(),
            plan_type: None,
            settings: None,
        };
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_organization_serialization() {
        let org = Organization {
            id: Uuid::nil(),
            name: "Test Org".to_string(),
            slug: "test-org".to_string(),
            billing_email: "test@test.com".to_string(),
            plan_type: PlanType::Starter,
            max_users: 25,
            max_devices: 100,
            max_groups: 20,
            settings: json!({"feature": true}),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&org).unwrap();
        assert!(json.contains("\"plan_type\":\"starter\""));
        assert!(json.contains("\"max_users\":25"));
        assert!(json.contains("\"billing_email\":\"test@test.com\""));
    }

    #[test]
    fn test_slug_regex() {
        assert!(SLUG_REGEX.is_match("acme-corp"));
        assert!(SLUG_REGEX.is_match("test123"));
        assert!(SLUG_REGEX.is_match("a1"));
        assert!(!SLUG_REGEX.is_match("Acme-Corp")); // uppercase
        assert!(!SLUG_REGEX.is_match("-acme")); // starts with hyphen
        assert!(!SLUG_REGEX.is_match("acme-")); // ends with hyphen
        assert!(!SLUG_REGEX.is_match("a")); // too short (need at least 2 chars per regex)
    }
}
