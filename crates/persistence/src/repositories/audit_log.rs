//! Audit log repository for database operations.
//!
//! Story 13.9: Audit Logging System

use domain::models::{
    ActorType, AuditActor, AuditLog, AuditMetadata, AuditResource,
    CreateAuditLogInput, FieldChange, ListAuditLogsQuery,
};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::AuditLogEntity;

/// Helper struct for building dynamic WHERE clauses from audit log filters.
/// Tracks conditions and parameter positions to avoid code duplication.
struct AuditLogFilterBuilder {
    conditions: Vec<String>,
    param_count: i32,
}

impl AuditLogFilterBuilder {
    /// Build filter conditions from a query.
    /// Returns the builder with WHERE clause and parameter count.
    fn build(query: &ListAuditLogsQuery) -> Self {
        let mut conditions = vec!["organization_id = $1".to_string()];
        let mut param_count = 1;

        if query.actor_id.is_some() {
            param_count += 1;
            conditions.push(format!("actor_id = ${}", param_count));
        }

        if query.action.is_some() {
            param_count += 1;
            conditions.push(format!("action = ${}", param_count));
        }

        if query.resource_type.is_some() {
            param_count += 1;
            conditions.push(format!("resource_type = ${}", param_count));
        }

        if query.resource_id.is_some() {
            param_count += 1;
            conditions.push(format!("resource_id = ${}", param_count));
        }

        if query.from.is_some() {
            param_count += 1;
            conditions.push(format!("timestamp >= ${}", param_count));
        }

        if query.to.is_some() {
            param_count += 1;
            conditions.push(format!("timestamp <= ${}", param_count));
        }

        Self { conditions, param_count }
    }

    /// Get the WHERE clause as a string.
    fn where_clause(&self) -> String {
        self.conditions.join(" AND ")
    }

    /// Get the current parameter count.
    fn param_count(&self) -> i32 {
        self.param_count
    }
}

/// Macro to bind query filter parameters to a SQLx builder.
/// This avoids code duplication for binding optional query parameters.
macro_rules! bind_query_filters {
    ($builder:expr, $query:expr) => {{
        let mut b = $builder;
        if let Some(ref actor_id) = $query.actor_id {
            b = b.bind(actor_id);
        }
        if let Some(ref action) = $query.action {
            b = b.bind(action);
        }
        if let Some(ref resource_type) = $query.resource_type {
            b = b.bind(resource_type);
        }
        if let Some(ref resource_id) = $query.resource_id {
            b = b.bind(resource_id);
        }
        if let Some(ref from) = $query.from {
            b = b.bind(from);
        }
        if let Some(ref to) = $query.to {
            b = b.bind(to);
        }
        b
    }};
}

/// Repository for audit log database operations.
#[derive(Clone)]
pub struct AuditLogRepository {
    pool: PgPool,
}

impl AuditLogRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new audit log entry.
    pub async fn insert(&self, input: CreateAuditLogInput) -> Result<AuditLog, sqlx::Error> {
        let changes_json = input.changes.as_ref().map(|changes| {
            serde_json::to_value(changes).unwrap_or(JsonValue::Null)
        });

        let metadata_json = if input.ip_address.is_some() || input.user_agent.is_some() || input.request_id.is_some() {
            Some(serde_json::json!({
                "ip_address": input.ip_address.map(|ip| ip.to_string()),
                "user_agent": input.user_agent,
                "request_id": input.request_id
            }))
        } else {
            None
        };

        let entity = sqlx::query_as::<_, AuditLogEntity>(
            r#"
            INSERT INTO audit_logs (
                organization_id, actor_id, actor_type, actor_email, action,
                resource_type, resource_id, resource_name, changes, metadata,
                ip_address, user_agent
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::inet, $12)
            RETURNING id, organization_id, timestamp, actor_id, actor_type, actor_email,
                      action, resource_type, resource_id, resource_name, changes, metadata,
                      ip_address::text, user_agent, created_at
            "#,
        )
        .bind(input.organization_id)
        .bind(input.actor_id)
        .bind(input.actor_type.to_string())
        .bind(&input.actor_email)
        .bind(input.action.to_string())
        .bind(&input.resource_type)
        .bind(&input.resource_id)
        .bind(&input.resource_name)
        .bind(changes_json)
        .bind(metadata_json)
        .bind(input.ip_address.map(|ip| ip.to_string()))
        .bind(&input.user_agent)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity_to_domain(entity))
    }

    /// Insert audit log entry asynchronously (fire and forget).
    /// Uses tokio::spawn to avoid blocking the request.
    pub fn insert_async(&self, input: CreateAuditLogInput) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let repo = AuditLogRepository::new(pool);
            if let Err(e) = repo.insert(input).await {
                tracing::error!("Failed to insert audit log: {}", e);
            }
        });
    }

    /// Find audit log by ID.
    pub async fn find_by_id(
        &self,
        org_id: Uuid,
        id: Uuid,
    ) -> Result<Option<AuditLog>, sqlx::Error> {
        let entity = sqlx::query_as::<_, AuditLogEntity>(
            r#"
            SELECT id, organization_id, timestamp, actor_id, actor_type, actor_email,
                   action, resource_type, resource_id, resource_name, changes, metadata,
                   ip_address::text, user_agent, created_at
            FROM audit_logs
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(entity_to_domain))
    }

    /// List audit logs with pagination and filtering.
    pub async fn list(
        &self,
        org_id: Uuid,
        query: &ListAuditLogsQuery,
    ) -> Result<(Vec<AuditLog>, i64), sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
        let offset = ((page - 1) * per_page) as i64;

        // Build filter conditions using helper
        let filter = AuditLogFilterBuilder::build(query);
        let where_clause = filter.where_clause();
        let param_count = filter.param_count();

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM audit_logs WHERE {}",
            where_clause
        );

        let count_builder = sqlx::query_scalar::<_, i64>(&count_query).bind(org_id);
        let count_builder = bind_query_filters!(count_builder, query);
        let total: i64 = count_builder.fetch_one(&self.pool).await?;

        // Get audit logs
        let list_query = format!(
            r#"
            SELECT id, organization_id, timestamp, actor_id, actor_type, actor_email,
                   action, resource_type, resource_id, resource_name, changes, metadata,
                   ip_address::text, user_agent, created_at
            FROM audit_logs
            WHERE {}
            ORDER BY timestamp DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_count + 1,
            param_count + 2
        );

        let list_builder = sqlx::query_as::<_, AuditLogEntity>(&list_query).bind(org_id);
        let list_builder = bind_query_filters!(list_builder, query);
        let entities = list_builder
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let logs = entities.into_iter().map(entity_to_domain).collect();

        Ok((logs, total))
    }

    /// Count audit logs for an organization.
    pub async fn count(&self, org_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Get audit logs for export (up to max_records).
    pub async fn list_for_export(
        &self,
        org_id: Uuid,
        query: &ListAuditLogsQuery,
        max_records: i64,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        // Build filter conditions using helper
        let filter = AuditLogFilterBuilder::build(query);
        let where_clause = filter.where_clause();
        let param_count = filter.param_count();

        let list_query = format!(
            r#"
            SELECT id, organization_id, timestamp, actor_id, actor_type, actor_email,
                   action, resource_type, resource_id, resource_name, changes, metadata,
                   ip_address::text, user_agent, created_at
            FROM audit_logs
            WHERE {}
            ORDER BY timestamp DESC
            LIMIT ${}
            "#,
            where_clause,
            param_count + 1
        );

        let list_builder = sqlx::query_as::<_, AuditLogEntity>(&list_query).bind(org_id);
        let list_builder = bind_query_filters!(list_builder, query);
        let entities = list_builder
            .bind(max_records)
            .fetch_all(&self.pool)
            .await?;

        let logs = entities.into_iter().map(entity_to_domain).collect();

        Ok(logs)
    }
}

/// Convert entity to domain model.
fn entity_to_domain(entity: AuditLogEntity) -> AuditLog {
    let actor_type = entity.actor_type.parse::<ActorType>().unwrap_or(ActorType::System);

    let changes: Option<HashMap<String, FieldChange>> = entity.changes.and_then(|json| {
        serde_json::from_value(json).ok()
    });

    let metadata = if entity.ip_address.is_some() || entity.user_agent.is_some() || entity.metadata.is_some() {
        Some(AuditMetadata {
            ip_address: entity.ip_address,
            user_agent: entity.user_agent,
            request_id: entity.metadata.as_ref().and_then(|m| {
                m.get("request_id").and_then(|v| v.as_str().map(String::from))
            }),
            extra: entity.metadata,
        })
    } else {
        None
    };

    AuditLog {
        id: entity.id,
        organization_id: entity.organization_id,
        timestamp: entity.timestamp,
        actor: AuditActor {
            id: entity.actor_id,
            actor_type,
            email: entity.actor_email,
        },
        action: entity.action,
        resource: AuditResource {
            resource_type: entity.resource_type,
            id: entity.resource_id,
            name: entity.resource_name,
        },
        changes,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_entity_to_domain_conversion() {
        let entity = AuditLogEntity {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: Some(Uuid::new_v4()),
            actor_type: "user".to_string(),
            actor_email: Some("admin@example.com".to_string()),
            action: "device.assign".to_string(),
            resource_type: "device".to_string(),
            resource_id: Some("device-123".to_string()),
            resource_name: Some("Field Tablet 1".to_string()),
            changes: Some(serde_json::json!({
                "assigned_user_id": {
                    "old": null,
                    "new": "user-456"
                }
            })),
            metadata: Some(serde_json::json!({
                "request_id": "req-123"
            })),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            created_at: Utc::now(),
        };

        let log = entity_to_domain(entity);

        assert_eq!(log.actor.actor_type, ActorType::User);
        assert_eq!(log.action, "device.assign");
        assert_eq!(log.resource.resource_type, "device");
        assert!(log.changes.is_some());
        assert!(log.metadata.is_some());
    }
}
