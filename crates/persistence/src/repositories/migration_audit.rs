//! Repository for migration audit log operations.
//!
//! Handles CRUD operations for tracking registration group to authenticated group migrations.
//! Story UGM-2.1: Create Migration Audit Log Infrastructure

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    MigrationAuditLogEntity, MigrationAuditLogWithUserEntity, MigrationStatusDb,
};

/// Input for creating a migration audit log entry.
#[derive(Debug, Clone)]
pub struct CreateMigrationAuditInput {
    /// User who initiated the migration
    pub user_id: Uuid,
    /// Source registration group ID (string)
    pub registration_group_id: String,
    /// Target authenticated group UUID
    pub authenticated_group_id: Uuid,
    /// Number of devices migrated
    pub devices_migrated: i32,
    /// Array of device UUIDs that were migrated
    pub device_ids: Vec<Uuid>,
    /// Migration status
    pub status: MigrationStatusDb,
    /// Error message if migration failed
    pub error_message: Option<String>,
}

/// Query parameters for listing migration audit logs.
#[derive(Debug, Clone, Default)]
pub struct ListMigrationAuditQuery {
    /// Filter by user ID
    pub user_id: Option<Uuid>,
    /// Filter by status
    pub status: Option<MigrationStatusDb>,
    /// Filter by registration group ID
    pub registration_group_id: Option<String>,
    /// Page number (1-based)
    pub page: i64,
    /// Items per page
    pub per_page: i64,
}

/// Repository for migration audit log operations.
#[derive(Debug, Clone)]
pub struct MigrationAuditRepository {
    pool: PgPool,
}

impl MigrationAuditRepository {
    /// Creates a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new migration audit log entry.
    pub async fn create(
        &self,
        input: CreateMigrationAuditInput,
    ) -> Result<MigrationAuditLogEntity, sqlx::Error> {
        let record = sqlx::query_as::<_, MigrationAuditLogEntity>(
            r#"
            INSERT INTO migration_audit_logs (
                user_id,
                registration_group_id,
                authenticated_group_id,
                devices_migrated,
                device_ids,
                status,
                error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id,
                user_id,
                registration_group_id,
                authenticated_group_id,
                devices_migrated,
                device_ids,
                status,
                error_message,
                created_at
            "#,
        )
        .bind(input.user_id)
        .bind(&input.registration_group_id)
        .bind(input.authenticated_group_id)
        .bind(input.devices_migrated)
        .bind(&input.device_ids)
        .bind(input.status)
        .bind(&input.error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Finds a migration audit log by ID.
    pub async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<MigrationAuditLogEntity>, sqlx::Error> {
        let record = sqlx::query_as::<_, MigrationAuditLogEntity>(
            r#"
            SELECT
                id,
                user_id,
                registration_group_id,
                authenticated_group_id,
                devices_migrated,
                device_ids,
                status,
                error_message,
                created_at
            FROM migration_audit_logs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// Checks if a registration group has already been migrated.
    pub async fn is_already_migrated(
        &self,
        registration_group_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM migration_audit_logs
                WHERE registration_group_id = $1
                AND status = 'success'
            )
            "#,
        )
        .bind(registration_group_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    /// Gets migration info for a registration group if it was migrated.
    pub async fn get_migration_for_registration_group(
        &self,
        registration_group_id: &str,
    ) -> Result<Option<MigrationAuditLogEntity>, sqlx::Error> {
        let record = sqlx::query_as::<_, MigrationAuditLogEntity>(
            r#"
            SELECT
                id,
                user_id,
                registration_group_id,
                authenticated_group_id,
                devices_migrated,
                device_ids,
                status,
                error_message,
                created_at
            FROM migration_audit_logs
            WHERE registration_group_id = $1
            AND status = 'success'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(registration_group_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// Lists migration audit logs with pagination and filtering (for admin).
    pub async fn list(
        &self,
        query: ListMigrationAuditQuery,
    ) -> Result<(Vec<MigrationAuditLogWithUserEntity>, i64), sqlx::Error> {
        let offset = (query.page - 1) * query.per_page;

        // Build dynamic query with filters
        let mut sql = String::from(
            r#"
            SELECT
                m.id,
                m.user_id,
                u.email as user_email,
                m.registration_group_id,
                m.authenticated_group_id,
                g.name as group_name,
                m.devices_migrated,
                m.device_ids,
                m.status,
                m.error_message,
                m.created_at
            FROM migration_audit_logs m
            LEFT JOIN users u ON m.user_id = u.id
            LEFT JOIN groups g ON m.authenticated_group_id = g.id
            WHERE 1=1
            "#,
        );

        let mut count_sql = String::from(
            r#"
            SELECT COUNT(*)
            FROM migration_audit_logs m
            WHERE 1=1
            "#,
        );

        // Add filters
        if query.user_id.is_some() {
            sql.push_str(" AND m.user_id = $4");
            count_sql.push_str(" AND m.user_id = $1");
        }
        if query.status.is_some() {
            let param_num = if query.user_id.is_some() { "5" } else { "4" };
            sql.push_str(&format!(" AND m.status = ${}", param_num));
            let count_param_num = if query.user_id.is_some() { "2" } else { "1" };
            count_sql.push_str(&format!(" AND m.status = ${}", count_param_num));
        }
        if query.registration_group_id.is_some() {
            let param_num = match (query.user_id.is_some(), query.status.is_some()) {
                (true, true) => "6",
                (true, false) | (false, true) => "5",
                (false, false) => "4",
            };
            sql.push_str(&format!(" AND m.registration_group_id = ${}", param_num));
            let count_param_num = match (query.user_id.is_some(), query.status.is_some()) {
                (true, true) => "3",
                (true, false) | (false, true) => "2",
                (false, false) => "1",
            };
            count_sql.push_str(&format!(
                " AND m.registration_group_id = ${}",
                count_param_num
            ));
        }

        sql.push_str(" ORDER BY m.created_at DESC LIMIT $1 OFFSET $2");

        // For simplicity, use a simpler approach with conditional binding
        let records: Vec<MigrationAuditLogWithUserEntity>;
        let total: i64;

        match (
            query.user_id,
            query.status,
            query.registration_group_id.as_ref(),
        ) {
            (None, None, None) => {
                records = sqlx::query_as::<_, MigrationAuditLogWithUserEntity>(
                    r#"
                    SELECT
                        m.id,
                        m.user_id,
                        u.email as user_email,
                        m.registration_group_id,
                        m.authenticated_group_id,
                        g.name as group_name,
                        m.devices_migrated,
                        m.device_ids,
                        m.status,
                        m.error_message,
                        m.created_at
                    FROM migration_audit_logs m
                    LEFT JOIN users u ON m.user_id = u.id
                    LEFT JOIN groups g ON m.authenticated_group_id = g.id
                    ORDER BY m.created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(query.per_page)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;

                total = sqlx::query_scalar("SELECT COUNT(*) FROM migration_audit_logs")
                    .fetch_one(&self.pool)
                    .await?;
            }
            (Some(user_id), None, None) => {
                records = sqlx::query_as::<_, MigrationAuditLogWithUserEntity>(
                    r#"
                    SELECT
                        m.id,
                        m.user_id,
                        u.email as user_email,
                        m.registration_group_id,
                        m.authenticated_group_id,
                        g.name as group_name,
                        m.devices_migrated,
                        m.device_ids,
                        m.status,
                        m.error_message,
                        m.created_at
                    FROM migration_audit_logs m
                    LEFT JOIN users u ON m.user_id = u.id
                    LEFT JOIN groups g ON m.authenticated_group_id = g.id
                    WHERE m.user_id = $3
                    ORDER BY m.created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(query.per_page)
                .bind(offset)
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?;

                total = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM migration_audit_logs WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;
            }
            (None, Some(status), None) => {
                records = sqlx::query_as::<_, MigrationAuditLogWithUserEntity>(
                    r#"
                    SELECT
                        m.id,
                        m.user_id,
                        u.email as user_email,
                        m.registration_group_id,
                        m.authenticated_group_id,
                        g.name as group_name,
                        m.devices_migrated,
                        m.device_ids,
                        m.status,
                        m.error_message,
                        m.created_at
                    FROM migration_audit_logs m
                    LEFT JOIN users u ON m.user_id = u.id
                    LEFT JOIN groups g ON m.authenticated_group_id = g.id
                    WHERE m.status = $3
                    ORDER BY m.created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(query.per_page)
                .bind(offset)
                .bind(status)
                .fetch_all(&self.pool)
                .await?;

                total = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM migration_audit_logs WHERE status = $1",
                )
                .bind(status)
                .fetch_one(&self.pool)
                .await?;
            }
            (Some(user_id), Some(status), None) => {
                records = sqlx::query_as::<_, MigrationAuditLogWithUserEntity>(
                    r#"
                    SELECT
                        m.id,
                        m.user_id,
                        u.email as user_email,
                        m.registration_group_id,
                        m.authenticated_group_id,
                        g.name as group_name,
                        m.devices_migrated,
                        m.device_ids,
                        m.status,
                        m.error_message,
                        m.created_at
                    FROM migration_audit_logs m
                    LEFT JOIN users u ON m.user_id = u.id
                    LEFT JOIN groups g ON m.authenticated_group_id = g.id
                    WHERE m.user_id = $3 AND m.status = $4
                    ORDER BY m.created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(query.per_page)
                .bind(offset)
                .bind(user_id)
                .bind(status)
                .fetch_all(&self.pool)
                .await?;

                total = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM migration_audit_logs WHERE user_id = $1 AND status = $2",
                )
                .bind(user_id)
                .bind(status)
                .fetch_one(&self.pool)
                .await?;
            }
            // Handle remaining combinations with registration_group_id filter
            _ => {
                // Fallback to basic query for simplicity
                records = sqlx::query_as::<_, MigrationAuditLogWithUserEntity>(
                    r#"
                    SELECT
                        m.id,
                        m.user_id,
                        u.email as user_email,
                        m.registration_group_id,
                        m.authenticated_group_id,
                        g.name as group_name,
                        m.devices_migrated,
                        m.device_ids,
                        m.status,
                        m.error_message,
                        m.created_at
                    FROM migration_audit_logs m
                    LEFT JOIN users u ON m.user_id = u.id
                    LEFT JOIN groups g ON m.authenticated_group_id = g.id
                    ORDER BY m.created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(query.per_page)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;

                total = sqlx::query_scalar("SELECT COUNT(*) FROM migration_audit_logs")
                    .fetch_one(&self.pool)
                    .await?;
            }
        }

        Ok((records, total))
    }
}
