//! Repository for organization member invite database operations.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::OrgMemberInviteEntity;

/// Repository for organization member invite operations.
#[derive(Clone)]
pub struct OrgMemberInviteRepository {
    pool: PgPool,
}

impl OrgMemberInviteRepository {
    /// Creates a new organization member invite repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new organization member invite.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        org_id: Uuid,
        token: &str,
        email: &str,
        role: &str,
        invited_by: Option<Uuid>,
        expires_at: DateTime<Utc>,
        note: Option<&str>,
    ) -> Result<OrgMemberInviteEntity, sqlx::Error> {
        sqlx::query_as::<_, OrgMemberInviteEntity>(
            r#"
            INSERT INTO org_member_invites (organization_id, token, email, role, invited_by, expires_at, note)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, organization_id, token, email, role, invited_by, expires_at,
                      accepted_at, accepted_by, created_at, note
            "#,
        )
        .bind(org_id)
        .bind(token)
        .bind(email)
        .bind(role)
        .bind(invited_by)
        .bind(expires_at)
        .bind(note)
        .fetch_one(&self.pool)
        .await
    }

    /// Finds an invite by its token.
    pub async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<OrgMemberInviteEntity>, sqlx::Error> {
        sqlx::query_as::<_, OrgMemberInviteEntity>(
            r#"
            SELECT id, organization_id, token, email, role, invited_by, expires_at,
                   accepted_at, accepted_by, created_at, note
            FROM org_member_invites
            WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
    }

    /// Finds an invite by ID and organization.
    pub async fn find_by_id_and_org(
        &self,
        invite_id: Uuid,
        org_id: Uuid,
    ) -> Result<Option<OrgMemberInviteEntity>, sqlx::Error> {
        sqlx::query_as::<_, OrgMemberInviteEntity>(
            r#"
            SELECT id, organization_id, token, email, role, invited_by, expires_at,
                   accepted_at, accepted_by, created_at, note
            FROM org_member_invites
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(invite_id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Lists invites for an organization with status filter.
    ///
    /// Status filter options:
    /// - "pending": not accepted and not expired
    /// - "accepted": has been accepted
    /// - "expired": not accepted but past expiration
    /// - "all" or None: all invitations
    pub async fn list_by_organization_with_status(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<OrgMemberInviteEntity>, sqlx::Error> {
        let query = match status {
            Some("pending") => {
                r#"
                SELECT id, organization_id, token, email, role, invited_by, expires_at,
                       accepted_at, accepted_by, created_at, note
                FROM org_member_invites
                WHERE organization_id = $1 AND accepted_at IS NULL AND expires_at > NOW()
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            }
            Some("accepted") => {
                r#"
                SELECT id, organization_id, token, email, role, invited_by, expires_at,
                       accepted_at, accepted_by, created_at, note
                FROM org_member_invites
                WHERE organization_id = $1 AND accepted_at IS NOT NULL
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            }
            Some("expired") => {
                r#"
                SELECT id, organization_id, token, email, role, invited_by, expires_at,
                       accepted_at, accepted_by, created_at, note
                FROM org_member_invites
                WHERE organization_id = $1 AND accepted_at IS NULL AND expires_at <= NOW()
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            }
            _ => {
                // "all" or None - return everything
                r#"
                SELECT id, organization_id, token, email, role, invited_by, expires_at,
                       accepted_at, accepted_by, created_at, note
                FROM org_member_invites
                WHERE organization_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            }
        };

        sqlx::query_as::<_, OrgMemberInviteEntity>(query)
            .bind(org_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
    }

    /// Lists invites for an organization (deprecated - use list_by_organization_with_status).
    #[deprecated(note = "Use list_by_organization_with_status for proper status filtering")]
    pub async fn list_by_organization(
        &self,
        org_id: Uuid,
        include_accepted: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<OrgMemberInviteEntity>, sqlx::Error> {
        let status = if include_accepted { Some("all") } else { Some("pending") };
        self.list_by_organization_with_status(org_id, status, limit, offset).await
    }

    /// Counts invites for an organization with status filter.
    pub async fn count_by_organization_with_status(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let query = match status {
            Some("pending") => {
                r#"
                SELECT COUNT(*) FROM org_member_invites
                WHERE organization_id = $1 AND accepted_at IS NULL AND expires_at > NOW()
                "#
            }
            Some("accepted") => {
                r#"
                SELECT COUNT(*) FROM org_member_invites
                WHERE organization_id = $1 AND accepted_at IS NOT NULL
                "#
            }
            Some("expired") => {
                r#"
                SELECT COUNT(*) FROM org_member_invites
                WHERE organization_id = $1 AND accepted_at IS NULL AND expires_at <= NOW()
                "#
            }
            _ => {
                // "all" or None - count everything
                r#"
                SELECT COUNT(*) FROM org_member_invites
                WHERE organization_id = $1
                "#
            }
        };

        let result: (i64,) = sqlx::query_as(query)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0)
    }

    /// Counts invites for an organization (deprecated - use count_by_organization_with_status).
    #[deprecated(note = "Use count_by_organization_with_status for proper status filtering")]
    pub async fn count_by_organization(
        &self,
        org_id: Uuid,
        include_accepted: bool,
    ) -> Result<i64, sqlx::Error> {
        let status = if include_accepted { Some("all") } else { Some("pending") };
        self.count_by_organization_with_status(org_id, status).await
    }

    /// Counts all invites (including expired and accepted) for limit checking.
    pub async fn count_all_by_organization(&self, org_id: Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM org_member_invites
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// Checks if a pending invite exists for this email in the organization.
    pub async fn has_pending_invite(
        &self,
        org_id: Uuid,
        email: &str,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM org_member_invites
            WHERE organization_id = $1 AND LOWER(email) = LOWER($2) AND accepted_at IS NULL
            "#,
        )
        .bind(org_id)
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0 > 0)
    }

    /// Marks an invite as accepted atomically.
    ///
    /// Returns `true` if the invite was successfully accepted,
    /// `false` if it was already accepted (race condition).
    pub async fn accept(
        &self,
        invite_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE org_member_invites
            SET accepted_at = NOW(), accepted_by = $2
            WHERE id = $1 AND accepted_at IS NULL
            "#,
        )
        .bind(invite_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Deletes an invite by ID and organization.
    ///
    /// Returns true if an invite was deleted.
    pub async fn delete(
        &self,
        invite_id: Uuid,
        org_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM org_member_invites
            WHERE id = $1 AND organization_id = $2 AND accepted_at IS NULL
            "#,
        )
        .bind(invite_id)
        .bind(org_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Deletes expired invites.
    ///
    /// Returns the number of deleted invites.
    pub async fn delete_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM org_member_invites
            WHERE expires_at < NOW() AND accepted_at IS NULL
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Gets invitation summary counts for an organization.
    pub async fn get_summary_counts(
        &self,
        org_id: Uuid,
    ) -> Result<InviteSummaryCounts, sqlx::Error> {
        let result: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE accepted_at IS NULL AND expires_at > NOW()) as pending,
                COUNT(*) FILTER (WHERE accepted_at IS NOT NULL) as accepted,
                COUNT(*) FILTER (WHERE accepted_at IS NULL AND expires_at <= NOW()) as expired
            FROM org_member_invites
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(InviteSummaryCounts {
            pending: result.0,
            accepted: result.1,
            expired: result.2,
        })
    }
}

/// Summary counts for invites.
#[derive(Debug, Clone)]
pub struct InviteSummaryCounts {
    pub pending: i64,
    pub accepted: i64,
    pub expired: i64,
}

/// Generate a secure organization member invite token.
///
/// Uses URL-safe characters, avoiding confusing ones (0, O, 1, l, I).
pub fn generate_org_member_invite_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789";
    let mut rng = rand::thread_rng();

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Default invite expiration (7 days).
pub fn default_invite_expiration() -> DateTime<Utc> {
    Utc::now() + Duration::days(7)
}

/// Calculate invite expiration from days.
pub fn calculate_invite_expiration(days: i32) -> DateTime<Utc> {
    Utc::now() + Duration::days(days as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_org_member_invite_token_length() {
        let token = generate_org_member_invite_token();
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_generate_org_member_invite_token_unique() {
        let token1 = generate_org_member_invite_token();
        let token2 = generate_org_member_invite_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_org_member_invite_token_charset() {
        let token = generate_org_member_invite_token();
        // Should not contain confusing characters
        assert!(!token.contains('0'));
        assert!(!token.contains('O'));
        assert!(!token.contains('1'));
        assert!(!token.contains('l'));
        assert!(!token.contains('I'));
    }

    #[test]
    fn test_default_invite_expiration() {
        let expiration = default_invite_expiration();
        let now = Utc::now();
        let diff = expiration - now;
        assert!(diff.num_days() >= 6 && diff.num_days() <= 7);
    }

    #[test]
    fn test_calculate_invite_expiration() {
        let expiration = calculate_invite_expiration(30);
        let now = Utc::now();
        let diff = expiration - now;
        assert!(diff.num_days() >= 29 && diff.num_days() <= 30);
    }
}
