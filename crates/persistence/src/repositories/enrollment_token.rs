//! Enrollment token repository for database operations.

use chrono::{DateTime, Utc};
use domain::models::enrollment_token::{EnrollmentToken, ListEnrollmentTokensQuery};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::enrollment_token::EnrollmentTokenEntity;

/// Repository for enrollment token database operations.
#[derive(Clone)]
pub struct EnrollmentTokenRepository {
    pool: PgPool,
}

impl EnrollmentTokenRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new enrollment token.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        organization_id: Uuid,
        token: &str,
        token_prefix: &str,
        group_id: Option<&str>,
        policy_id: Option<Uuid>,
        max_uses: Option<i32>,
        expires_at: Option<DateTime<Utc>>,
        auto_assign_user_by_email: bool,
        created_by: Option<Uuid>,
    ) -> Result<EnrollmentToken, sqlx::Error> {
        let entity = sqlx::query_as::<_, EnrollmentTokenEntity>(
            r#"
            INSERT INTO enrollment_tokens (organization_id, token, token_prefix, group_id, policy_id, max_uses, expires_at, auto_assign_user_by_email, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, organization_id, token, token_prefix, group_id, policy_id, max_uses, current_uses, expires_at, auto_assign_user_by_email, created_by, created_at, revoked_at
            "#,
        )
        .bind(organization_id)
        .bind(token)
        .bind(token_prefix)
        .bind(group_id)
        .bind(policy_id)
        .bind(max_uses)
        .bind(expires_at)
        .bind(auto_assign_user_by_email)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity.into())
    }

    /// Find token by ID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<EnrollmentToken>, sqlx::Error> {
        let entity = sqlx::query_as::<_, EnrollmentTokenEntity>(
            r#"
            SELECT id, organization_id, token, token_prefix, group_id, policy_id, max_uses, current_uses, expires_at, auto_assign_user_by_email, created_by, created_at, revoked_at
            FROM enrollment_tokens
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Find token by token string.
    pub async fn find_by_token(&self, token: &str) -> Result<Option<EnrollmentToken>, sqlx::Error> {
        let entity = sqlx::query_as::<_, EnrollmentTokenEntity>(
            r#"
            SELECT id, organization_id, token, token_prefix, group_id, policy_id, max_uses, current_uses, expires_at, auto_assign_user_by_email, created_by, created_at, revoked_at
            FROM enrollment_tokens
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Find valid token by token string (not revoked, not expired, not exhausted).
    pub async fn find_valid_token(
        &self,
        token: &str,
    ) -> Result<Option<EnrollmentToken>, sqlx::Error> {
        let entity = sqlx::query_as::<_, EnrollmentTokenEntity>(
            r#"
            SELECT id, organization_id, token, token_prefix, group_id, policy_id, max_uses, current_uses, expires_at, auto_assign_user_by_email, created_by, created_at, revoked_at
            FROM enrollment_tokens
            WHERE token = $1
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
              AND (max_uses IS NULL OR current_uses < max_uses)
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity.map(Into::into))
    }

    /// Revoke a token (soft delete).
    pub async fn revoke(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE enrollment_tokens
            SET revoked_at = NOW()
            WHERE id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Increment usage count.
    pub async fn increment_usage(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE enrollment_tokens
            SET current_uses = current_uses + 1
            WHERE id = $1
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
              AND (max_uses IS NULL OR current_uses < max_uses)
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List tokens for organization with pagination.
    pub async fn list(
        &self,
        organization_id: Uuid,
        query: &ListEnrollmentTokensQuery,
    ) -> Result<(Vec<EnrollmentToken>, i64), sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
        let offset = ((page - 1) * per_page) as i64;
        let include_revoked = query.include_revoked.unwrap_or(false);
        let include_expired = query.include_expired.unwrap_or(false);

        // Build WHERE clause based on filters
        let (where_clause, count_clause) = if include_revoked && include_expired {
            (
                "WHERE organization_id = $1".to_string(),
                "WHERE organization_id = $1".to_string(),
            )
        } else if include_revoked {
            (
                "WHERE organization_id = $1 AND (expires_at IS NULL OR expires_at > NOW())"
                    .to_string(),
                "WHERE organization_id = $1 AND (expires_at IS NULL OR expires_at > NOW())"
                    .to_string(),
            )
        } else if include_expired {
            (
                "WHERE organization_id = $1 AND revoked_at IS NULL".to_string(),
                "WHERE organization_id = $1 AND revoked_at IS NULL".to_string(),
            )
        } else {
            (
                "WHERE organization_id = $1 AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > NOW())".to_string(),
                "WHERE organization_id = $1 AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > NOW())".to_string(),
            )
        };

        // Get total count
        let count_query = format!("SELECT COUNT(*) FROM enrollment_tokens {}", count_clause);
        let total: i64 = sqlx::query_scalar(&count_query)
            .bind(organization_id)
            .fetch_one(&self.pool)
            .await?;

        // Get tokens
        let select_query = format!(
            r#"
            SELECT id, organization_id, token, token_prefix, group_id, policy_id, max_uses, current_uses, expires_at, auto_assign_user_by_email, created_by, created_at, revoked_at
            FROM enrollment_tokens
            {}
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            where_clause
        );

        let entities = sqlx::query_as::<_, EnrollmentTokenEntity>(&select_query)
            .bind(organization_id)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let tokens = entities.into_iter().map(Into::into).collect();

        Ok((tokens, total))
    }

    /// Count active tokens for organization.
    pub async fn count_active(&self, organization_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM enrollment_tokens
            WHERE organization_id = $1
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
              AND (max_uses IS NULL OR current_uses < max_uses)
            "#,
        )
        .bind(organization_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_enrollment_token_repository_new() {
        // This is a compile-time test - repository should be constructable
        // Actual DB tests require integration test setup
    }
}
