//! Unlock request repository for database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{UnlockRequestEntity, UnlockRequestStatusDb, UnlockRequestWithDetailsEntity};
use crate::metrics::QueryTimer;

/// Repository for unlock request-related database operations.
#[derive(Clone)]
pub struct UnlockRequestRepository {
    pool: PgPool,
}

impl UnlockRequestRepository {
    /// Creates a new UnlockRequestRepository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Create a new unlock request.
    pub async fn create(
        &self,
        device_id: Uuid,
        setting_key: &str,
        requested_by: Uuid,
        reason: Option<&str>,
    ) -> Result<UnlockRequestEntity, sqlx::Error> {
        let timer = QueryTimer::new("create_unlock_request");
        let result = sqlx::query_as::<_, UnlockRequestEntity>(
            r#"
            INSERT INTO unlock_requests (device_id, setting_key, requested_by, reason)
            VALUES ($1, $2, $3, $4)
            RETURNING id, device_id, setting_key, requested_by, status, reason,
                      responded_by, response_note, created_at, updated_at, expires_at, responded_at
            "#,
        )
        .bind(device_id)
        .bind(setting_key)
        .bind(requested_by)
        .bind(reason)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find an unlock request by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UnlockRequestEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_unlock_request_by_id");
        let result = sqlx::query_as::<_, UnlockRequestEntity>(
            r#"
            SELECT id, device_id, setting_key, requested_by, status, reason,
                   responded_by, response_note, created_at, updated_at, expires_at, responded_at
            FROM unlock_requests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find a pending request for a device and setting.
    pub async fn find_pending_for_device_setting(
        &self,
        device_id: Uuid,
        setting_key: &str,
    ) -> Result<Option<UnlockRequestEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_pending_unlock_request");
        let result = sqlx::query_as::<_, UnlockRequestEntity>(
            r#"
            SELECT id, device_id, setting_key, requested_by, status, reason,
                   responded_by, response_note, created_at, updated_at, expires_at, responded_at
            FROM unlock_requests
            WHERE device_id = $1 AND setting_key = $2 AND status = 'pending'
            "#,
        )
        .bind(device_id)
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// List unlock requests for a group (devices in the group).
    pub async fn list_for_group(
        &self,
        group_id: Uuid,
        status_filter: Option<UnlockRequestStatusDb>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UnlockRequestWithDetailsEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_unlock_requests_for_group");

        // Note: This query joins with group_memberships to find devices in the group
        // and includes user/device/setting details
        let result = if let Some(status) = status_filter {
            sqlx::query_as::<_, UnlockRequestWithDetailsEntity>(
                r#"
                SELECT ur.id, ur.device_id, d.display_name as device_display_name,
                       ur.setting_key, sd.display_name as setting_display_name,
                       ur.requested_by, u1.display_name as requester_display_name,
                       ur.status, ur.reason, ur.responded_by,
                       u2.display_name as responder_display_name,
                       ur.response_note, ur.created_at, ur.updated_at,
                       ur.expires_at, ur.responded_at
                FROM unlock_requests ur
                JOIN devices d ON ur.device_id = d.device_id
                JOIN setting_definitions sd ON ur.setting_key = sd.key
                JOIN users u1 ON ur.requested_by = u1.id
                LEFT JOIN users u2 ON ur.responded_by = u2.id
                JOIN group_memberships gm ON d.owner_user_id = gm.user_id
                WHERE gm.group_id = $1 AND ur.status = $2
                ORDER BY ur.created_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(group_id)
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, UnlockRequestWithDetailsEntity>(
                r#"
                SELECT ur.id, ur.device_id, d.display_name as device_display_name,
                       ur.setting_key, sd.display_name as setting_display_name,
                       ur.requested_by, u1.display_name as requester_display_name,
                       ur.status, ur.reason, ur.responded_by,
                       u2.display_name as responder_display_name,
                       ur.response_note, ur.created_at, ur.updated_at,
                       ur.expires_at, ur.responded_at
                FROM unlock_requests ur
                JOIN devices d ON ur.device_id = d.device_id
                JOIN setting_definitions sd ON ur.setting_key = sd.key
                JOIN users u1 ON ur.requested_by = u1.id
                LEFT JOIN users u2 ON ur.responded_by = u2.id
                JOIN group_memberships gm ON d.owner_user_id = gm.user_id
                WHERE gm.group_id = $1
                ORDER BY ur.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(group_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        timer.record();
        result
    }

    /// Count unlock requests for a group.
    pub async fn count_for_group(
        &self,
        group_id: Uuid,
        status_filter: Option<UnlockRequestStatusDb>,
    ) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_unlock_requests_for_group");
        let result = if let Some(status) = status_filter {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)
                FROM unlock_requests ur
                JOIN devices d ON ur.device_id = d.device_id
                JOIN group_memberships gm ON d.owner_user_id = gm.user_id
                WHERE gm.group_id = $1 AND ur.status = $2
                "#,
            )
            .bind(group_id)
            .bind(status)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)
                FROM unlock_requests ur
                JOIN devices d ON ur.device_id = d.device_id
                JOIN group_memberships gm ON d.owner_user_id = gm.user_id
                WHERE gm.group_id = $1
                "#,
            )
            .bind(group_id)
            .fetch_one(&self.pool)
            .await
        };
        timer.record();
        result
    }

    /// Respond to an unlock request (approve or deny).
    pub async fn respond(
        &self,
        id: Uuid,
        status: UnlockRequestStatusDb,
        responded_by: Uuid,
        response_note: Option<&str>,
    ) -> Result<Option<UnlockRequestEntity>, sqlx::Error> {
        let timer = QueryTimer::new("respond_to_unlock_request");
        let result = sqlx::query_as::<_, UnlockRequestEntity>(
            r#"
            UPDATE unlock_requests
            SET status = $2, responded_by = $3, response_note = $4, responded_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status = 'pending'
            RETURNING id, device_id, setting_key, requested_by, status, reason,
                      responded_by, response_note, created_at, updated_at, expires_at, responded_at
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(responded_by)
        .bind(response_note)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Expire old pending requests.
    pub async fn expire_old_requests(&self) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("expire_old_unlock_requests");
        let result = sqlx::query(
            r#"
            UPDATE unlock_requests
            SET status = 'expired', updated_at = NOW()
            WHERE status = 'pending' AND expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() as i64);
        timer.record();
        result
    }

    /// List unlock requests for an organization (devices in the organization).
    pub async fn list_for_organization(
        &self,
        org_id: Uuid,
        status_filter: Option<UnlockRequestStatusDb>,
        device_id_filter: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UnlockRequestWithDetailsEntity>, sqlx::Error> {
        let timer = QueryTimer::new("list_unlock_requests_for_org");

        let result = sqlx::query_as::<_, UnlockRequestWithDetailsEntity>(
            r#"
            SELECT ur.id, ur.device_id, d.display_name as device_display_name,
                   ur.setting_key, COALESCE(sd.display_name, ur.setting_key) as setting_display_name,
                   ur.requested_by, u1.display_name as requester_display_name,
                   ur.status, ur.reason, ur.responded_by,
                   u2.display_name as responder_display_name,
                   ur.response_note, ur.created_at, ur.updated_at,
                   ur.expires_at, ur.responded_at
            FROM unlock_requests ur
            JOIN devices d ON ur.device_id = d.device_id
            LEFT JOIN setting_definitions sd ON ur.setting_key = sd.key
            JOIN users u1 ON ur.requested_by = u1.id
            LEFT JOIN users u2 ON ur.responded_by = u2.id
            WHERE d.organization_id = $1
              AND ($2::unlock_request_status IS NULL OR ur.status = $2)
              AND ($3::UUID IS NULL OR ur.device_id = $3)
            ORDER BY ur.created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(org_id)
        .bind(status_filter)
        .bind(device_id_filter)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Count unlock requests for an organization.
    pub async fn count_for_organization(
        &self,
        org_id: Uuid,
        status_filter: Option<UnlockRequestStatusDb>,
        device_id_filter: Option<Uuid>,
    ) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("count_unlock_requests_for_org");
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM unlock_requests ur
            JOIN devices d ON ur.device_id = d.device_id
            WHERE d.organization_id = $1
              AND ($2::unlock_request_status IS NULL OR ur.status = $2)
              AND ($3::UUID IS NULL OR ur.device_id = $3)
            "#,
        )
        .bind(org_id)
        .bind(status_filter)
        .bind(device_id_filter)
        .fetch_one(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Find an unlock request by ID with organization scope.
    pub async fn find_by_id_for_organization(
        &self,
        id: Uuid,
        org_id: Uuid,
    ) -> Result<Option<UnlockRequestWithDetailsEntity>, sqlx::Error> {
        let timer = QueryTimer::new("find_unlock_request_by_id_for_org");
        let result = sqlx::query_as::<_, UnlockRequestWithDetailsEntity>(
            r#"
            SELECT ur.id, ur.device_id, d.display_name as device_display_name,
                   ur.setting_key, COALESCE(sd.display_name, ur.setting_key) as setting_display_name,
                   ur.requested_by, u1.display_name as requester_display_name,
                   ur.status, ur.reason, ur.responded_by,
                   u2.display_name as responder_display_name,
                   ur.response_note, ur.created_at, ur.updated_at,
                   ur.expires_at, ur.responded_at
            FROM unlock_requests ur
            JOIN devices d ON ur.device_id = d.device_id
            LEFT JOIN setting_definitions sd ON ur.setting_key = sd.key
            JOIN users u1 ON ur.requested_by = u1.id
            LEFT JOIN users u2 ON ur.responded_by = u2.id
            WHERE ur.id = $1 AND d.organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await;
        timer.record();
        result
    }

    /// Bulk respond to unlock requests with organization scope.
    pub async fn bulk_respond(
        &self,
        request_ids: &[Uuid],
        org_id: Uuid,
        status: UnlockRequestStatusDb,
        responded_by: Uuid,
        response_note: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let timer = QueryTimer::new("bulk_respond_unlock_requests");
        let result = sqlx::query(
            r#"
            UPDATE unlock_requests ur
            SET status = $3, responded_by = $4, response_note = $5, responded_at = NOW(), updated_at = NOW()
            FROM devices d
            WHERE ur.device_id = d.device_id
              AND ur.id = ANY($1)
              AND d.organization_id = $2
              AND ur.status = 'pending'
            "#,
        )
        .bind(request_ids)
        .bind(org_id)
        .bind(status)
        .bind(responded_by)
        .bind(response_note)
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected() as i64);
        timer.record();
        result
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_unlock_request_repository_new() {
        // This is a structural test - we can't test actual database operations without a test DB
    }
}
