//! Organization entity (database row mapping).

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

/// Database enum for plan_type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "plan_type", rename_all = "lowercase")]
pub enum PlanTypeDb {
    Free,
    Starter,
    Business,
    Enterprise,
}

impl From<PlanTypeDb> for domain::models::PlanType {
    fn from(db: PlanTypeDb) -> Self {
        match db {
            PlanTypeDb::Free => Self::Free,
            PlanTypeDb::Starter => Self::Starter,
            PlanTypeDb::Business => Self::Business,
            PlanTypeDb::Enterprise => Self::Enterprise,
        }
    }
}

impl From<domain::models::PlanType> for PlanTypeDb {
    fn from(domain: domain::models::PlanType) -> Self {
        match domain {
            domain::models::PlanType::Free => Self::Free,
            domain::models::PlanType::Starter => Self::Starter,
            domain::models::PlanType::Business => Self::Business,
            domain::models::PlanType::Enterprise => Self::Enterprise,
        }
    }
}

/// Database row mapping for the organizations table.
#[derive(Debug, Clone, FromRow)]
pub struct OrganizationEntity {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub billing_email: String,
    pub plan_type: PlanTypeDb,
    pub max_users: i32,
    pub max_devices: i32,
    pub max_groups: i32,
    pub settings: JsonValue,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Suspension fields
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspended_by: Option<Uuid>,
    pub suspension_reason: Option<String>,
}

impl From<OrganizationEntity> for domain::models::Organization {
    fn from(entity: OrganizationEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            slug: entity.slug,
            billing_email: entity.billing_email,
            plan_type: entity.plan_type.into(),
            max_users: entity.max_users,
            max_devices: entity.max_devices,
            max_groups: entity.max_groups,
            settings: entity.settings,
            is_active: entity.is_active,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            suspended_at: entity.suspended_at,
            suspended_by: entity.suspended_by,
            suspension_reason: entity.suspension_reason,
        }
    }
}

/// Organization with usage statistics for admin dashboard.
#[derive(Debug, Clone, FromRow)]
pub struct OrganizationWithUsageEntity {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub billing_email: String,
    pub plan_type: PlanTypeDb,
    pub max_users: i32,
    pub max_devices: i32,
    pub max_groups: i32,
    pub settings: JsonValue,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Suspension fields
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspended_by: Option<Uuid>,
    pub suspension_reason: Option<String>,
    // Usage statistics
    pub current_users: i64,
    pub current_devices: i64,
    pub current_groups: i64,
}

impl From<OrganizationWithUsageEntity> for domain::models::OrganizationWithUsage {
    fn from(entity: OrganizationWithUsageEntity) -> Self {
        Self {
            organization: domain::models::Organization {
                id: entity.id,
                name: entity.name,
                slug: entity.slug,
                billing_email: entity.billing_email,
                plan_type: entity.plan_type.into(),
                max_users: entity.max_users,
                max_devices: entity.max_devices,
                max_groups: entity.max_groups,
                settings: entity.settings,
                is_active: entity.is_active,
                created_at: entity.created_at,
                updated_at: entity.updated_at,
                suspended_at: entity.suspended_at,
                suspended_by: entity.suspended_by,
                suspension_reason: entity.suspension_reason,
            },
            current_users: entity.current_users,
            current_devices: entity.current_devices,
            current_groups: entity.current_groups,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_type_conversion() {
        assert_eq!(
            domain::models::PlanType::from(PlanTypeDb::Free),
            domain::models::PlanType::Free
        );
        assert_eq!(
            domain::models::PlanType::from(PlanTypeDb::Business),
            domain::models::PlanType::Business
        );
        assert_eq!(
            PlanTypeDb::from(domain::models::PlanType::Enterprise),
            PlanTypeDb::Enterprise
        );
    }
}
