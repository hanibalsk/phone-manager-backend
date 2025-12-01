//! Enrollment token entity for database operations.

use chrono::{DateTime, Utc};
use domain::models::enrollment_token::EnrollmentToken;
use sqlx::FromRow;
use uuid::Uuid;

/// Database entity for enrollment tokens.
#[derive(Debug, Clone, FromRow)]
pub struct EnrollmentTokenEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub token: String,
    pub token_prefix: String,
    pub group_id: Option<String>,
    pub policy_id: Option<Uuid>,
    pub max_uses: Option<i32>,
    pub current_uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_assign_user_by_email: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<EnrollmentTokenEntity> for EnrollmentToken {
    fn from(entity: EnrollmentTokenEntity) -> Self {
        EnrollmentToken {
            id: entity.id,
            organization_id: entity.organization_id,
            token: entity.token,
            token_prefix: entity.token_prefix,
            group_id: entity.group_id,
            policy_id: entity.policy_id,
            max_uses: entity.max_uses,
            current_uses: entity.current_uses,
            expires_at: entity.expires_at,
            auto_assign_user_by_email: entity.auto_assign_user_by_email,
            created_by: entity.created_by,
            created_at: entity.created_at,
            revoked_at: entity.revoked_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enrollment_token_entity_to_domain() {
        let now = Utc::now();
        let entity = EnrollmentTokenEntity {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            token: "enroll_abc123xyz".to_string(),
            token_prefix: "enroll_a".to_string(),
            group_id: Some("grp123".to_string()),
            policy_id: Some(Uuid::new_v4()),
            max_uses: Some(50),
            current_uses: 5,
            expires_at: Some(now),
            auto_assign_user_by_email: true,
            created_by: Some(Uuid::new_v4()),
            created_at: now,
            revoked_at: None,
        };

        let token: EnrollmentToken = entity.clone().into();
        assert_eq!(token.id, entity.id);
        assert_eq!(token.token, entity.token);
        assert_eq!(token.max_uses, Some(50));
        assert_eq!(token.current_uses, 5);
        assert!(token.auto_assign_user_by_email);
    }
}
