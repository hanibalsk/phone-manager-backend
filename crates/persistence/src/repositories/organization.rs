//! Organization repository for database operations.

use chrono::Utc;
use domain::models::{
    DeviceStatusCounts, DeviceUsageMetric, ListOrganizationsQuery, Organization,
    OrganizationUsageResponse, PlanType, UsageMetric,
};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::organization::{OrganizationEntity, PlanTypeDb};

/// Repository for organization database operations.
#[derive(Clone)]
pub struct OrganizationRepository {
    pool: PgPool,
}

impl OrganizationRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new organization.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        slug: &str,
        billing_email: &str,
        plan_type: PlanType,
        max_users: i32,
        max_devices: i32,
        max_groups: i32,
        settings: &JsonValue,
    ) -> Result<Organization, sqlx::Error> {
        let entity = sqlx::query_as::<_, OrganizationEntity>(
            r#"
            INSERT INTO organizations (name, slug, billing_email, plan_type, max_users, max_devices, max_groups, settings)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, slug, billing_email, plan_type, max_users, max_devices, max_groups, settings, is_active, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(slug)
        .bind(billing_email)
        .bind(PlanTypeDb::from(plan_type))
        .bind(max_users)
        .bind(max_devices)
        .bind(max_groups)
        .bind(settings)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity.into())
    }

    /// Find organization by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>, sqlx::Error> {
        let entity = sqlx::query_as::<_, OrganizationEntity>(
            r#"
            SELECT id, name, slug, billing_email, plan_type, max_users, max_devices, max_groups, settings, is_active, created_at, updated_at
            FROM organizations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Find organization by slug.
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, sqlx::Error> {
        let entity = sqlx::query_as::<_, OrganizationEntity>(
            r#"
            SELECT id, name, slug, billing_email, plan_type, max_users, max_devices, max_groups, settings, is_active, created_at, updated_at
            FROM organizations
            WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Check if slug is already taken.
    pub async fn slug_exists(&self, slug: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM organizations WHERE slug = $1)
            "#,
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Update organization.
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        billing_email: Option<&str>,
        plan_type: Option<PlanType>,
        max_users: Option<i32>,
        max_devices: Option<i32>,
        max_groups: Option<i32>,
        settings: Option<&JsonValue>,
    ) -> Result<Option<Organization>, sqlx::Error> {
        let entity = sqlx::query_as::<_, OrganizationEntity>(
            r#"
            UPDATE organizations
            SET
                name = COALESCE($2, name),
                billing_email = COALESCE($3, billing_email),
                plan_type = COALESCE($4, plan_type),
                max_users = COALESCE($5, max_users),
                max_devices = COALESCE($6, max_devices),
                max_groups = COALESCE($7, max_groups),
                settings = COALESCE($8, settings),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, slug, billing_email, plan_type, max_users, max_devices, max_groups, settings, is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(billing_email)
        .bind(plan_type.map(PlanTypeDb::from))
        .bind(max_users)
        .bind(max_devices)
        .bind(max_groups)
        .bind(settings)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Soft delete organization (set is_active = false).
    pub async fn soft_delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE organizations
            SET is_active = false, updated_at = NOW()
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List organizations with pagination and filtering.
    pub async fn list(
        &self,
        query: &ListOrganizationsQuery,
    ) -> Result<(Vec<Organization>, i64), sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
        let offset = ((page - 1) * per_page) as i64;

        // Build dynamic query based on filters
        let mut conditions = Vec::new();

        if let Some(is_active) = query.is_active {
            conditions.push(format!("is_active = {}", is_active));
        }

        if let Some(ref plan_type) = query.plan_type {
            conditions.push(format!("plan_type = '{}'", plan_type));
        }

        if let Some(ref search) = query.search {
            let search_escaped = search.replace("'", "''");
            conditions.push(format!(
                "(name ILIKE '%{}%' OR slug ILIKE '%{}%')",
                search_escaped, search_escaped
            ));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM organizations {}",
            where_clause
        );
        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await?;

        // Get organizations
        let list_query = format!(
            r#"
            SELECT id, name, slug, billing_email, plan_type, max_users, max_devices, max_groups, settings, is_active, created_at, updated_at
            FROM organizations
            {}
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            where_clause
        );

        let entities = sqlx::query_as::<_, OrganizationEntity>(&list_query)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let organizations = entities.into_iter().map(Into::into).collect();

        Ok((organizations, total))
    }

    /// Get organization usage statistics.
    pub async fn get_usage(&self, org_id: Uuid) -> Result<Option<OrganizationUsageResponse>, sqlx::Error> {
        // First check if org exists
        let org = self.find_by_id(org_id).await?;
        let org = match org {
            Some(o) => o,
            None => return Ok(None),
        };

        // Get current user count (org_users table will be created in story 13.2)
        // For now, return 0 as placeholder
        let current_users: i64 = 0;

        // Get current device count (devices with organization_id will be added later)
        // For now, return 0 as placeholder
        let current_devices: i64 = 0;

        // Get current group count (groups with organization_id will be added later)
        // For now, return 0 as placeholder
        let current_groups: i64 = 0;

        let users_percentage = if org.max_users > 0 {
            (current_users as f64 / org.max_users as f64) * 100.0
        } else {
            0.0
        };

        let devices_percentage = if org.max_devices > 0 {
            (current_devices as f64 / org.max_devices as f64) * 100.0
        } else {
            0.0
        };

        let groups_percentage = if org.max_groups > 0 {
            (current_groups as f64 / org.max_groups as f64) * 100.0
        } else {
            0.0
        };

        let period = Utc::now().format("%Y-%m").to_string();

        Ok(Some(OrganizationUsageResponse {
            organization_id: org_id,
            users: UsageMetric {
                current: current_users,
                max: org.max_users,
                percentage: users_percentage,
            },
            devices: DeviceUsageMetric {
                current: current_devices,
                max: org.max_devices,
                percentage: devices_percentage,
                by_status: DeviceStatusCounts::default(),
            },
            groups: UsageMetric {
                current: current_groups,
                max: org.max_groups,
                percentage: groups_percentage,
            },
            period,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_type_db_conversion() {
        assert_eq!(PlanTypeDb::from(PlanType::Free), PlanTypeDb::Free);
        assert_eq!(PlanType::from(PlanTypeDb::Business), PlanType::Business);
    }
}
