//! Audit logging service for tracking user and system actions.
//!
//! Provides convenient methods for creating audit log entries from route handlers.
//! Uses async insertion to avoid blocking request processing.

use crate::models::{ActorType, AuditAction, CreateAuditLogInput, FieldChange};
use serde_json::json;
use std::collections::HashMap;
use std::net::IpAddr;
use uuid::Uuid;

/// Builder for creating audit log entries with a fluent API.
///
/// This provides a more ergonomic way to create audit logs compared to
/// using `CreateAuditLogInput::new()` directly.
#[derive(Debug, Clone)]
pub struct AuditLogBuilder {
    organization_id: Uuid,
    actor_id: Option<Uuid>,
    actor_type: ActorType,
    actor_email: Option<String>,
    action: AuditAction,
    resource_type: String,
    resource_id: Option<String>,
    resource_name: Option<String>,
    changes: Option<HashMap<String, FieldChange>>,
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
    request_id: Option<String>,
}

impl AuditLogBuilder {
    /// Create a new audit log builder for a user action.
    pub fn user_action(org_id: Uuid, user_id: Uuid, action: AuditAction) -> Self {
        Self {
            organization_id: org_id,
            actor_id: Some(user_id),
            actor_type: ActorType::User,
            actor_email: None,
            action,
            resource_type: String::new(),
            resource_id: None,
            resource_name: None,
            changes: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
        }
    }

    /// Create a new audit log builder for a system action.
    pub fn system_action(org_id: Uuid, action: AuditAction) -> Self {
        Self {
            organization_id: org_id,
            actor_id: None,
            actor_type: ActorType::System,
            actor_email: None,
            action,
            resource_type: String::new(),
            resource_id: None,
            resource_name: None,
            changes: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
        }
    }

    /// Create a new audit log builder for an API key action.
    pub fn api_key_action(org_id: Uuid, api_key_id: Uuid, action: AuditAction) -> Self {
        Self {
            organization_id: org_id,
            actor_id: Some(api_key_id),
            actor_type: ActorType::ApiKey,
            actor_email: None,
            action,
            resource_type: String::new(),
            resource_id: None,
            resource_name: None,
            changes: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
        }
    }

    /// Set the actor's email.
    pub fn with_actor_email(mut self, email: impl Into<String>) -> Self {
        self.actor_email = Some(email.into());
        self
    }

    /// Set the resource being acted upon.
    pub fn on_resource(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        self.resource_type = resource_type.into();
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set just the resource type (when no ID is available).
    pub fn on_resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = resource_type.into();
        self
    }

    /// Set the resource name.
    pub fn with_resource_name(mut self, name: impl Into<String>) -> Self {
        self.resource_name = Some(name.into());
        self
    }

    /// Add a single field change with string values.
    pub fn with_change(
        mut self,
        field: impl Into<String>,
        old: Option<String>,
        new: Option<String>,
    ) -> Self {
        let changes = self.changes.get_or_insert_with(HashMap::new);
        changes.insert(
            field.into(),
            FieldChange::new(old.map(|v| json!(v)), new.map(|v| json!(v))),
        );
        self
    }

    /// Add a single field change with JSON values.
    pub fn with_json_change(
        mut self,
        field: impl Into<String>,
        old: Option<serde_json::Value>,
        new: Option<serde_json::Value>,
    ) -> Self {
        let changes = self.changes.get_or_insert_with(HashMap::new);
        changes.insert(field.into(), FieldChange::new(old, new));
        self
    }

    /// Add multiple field changes.
    pub fn with_changes(mut self, changes: HashMap<String, FieldChange>) -> Self {
        self.changes = Some(changes);
        self
    }

    /// Set the client IP address.
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Set the user agent string.
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Set the request ID for tracing.
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Build the CreateAuditLogInput.
    pub fn build(self) -> CreateAuditLogInput {
        CreateAuditLogInput::new(self.organization_id, self.action, self.resource_type)
            .with_resource_id_opt(self.resource_id)
            .with_resource_name_opt(self.resource_name)
            .with_changes_opt(self.changes)
            .with_actor(self.actor_id, self.actor_type, self.actor_email)
            .with_request_context(self.ip_address, self.user_agent, self.request_id)
    }
}

/// Extension trait for CreateAuditLogInput to support optional values.
trait CreateAuditLogInputExt {
    fn with_resource_id_opt(self, id: Option<String>) -> Self;
    fn with_resource_name_opt(self, name: Option<String>) -> Self;
    fn with_changes_opt(self, changes: Option<HashMap<String, FieldChange>>) -> Self;
    fn with_actor(self, id: Option<Uuid>, actor_type: ActorType, email: Option<String>) -> Self;
}

impl CreateAuditLogInputExt for CreateAuditLogInput {
    fn with_resource_id_opt(self, id: Option<String>) -> Self {
        match id {
            Some(id) => self.with_resource_id(id),
            None => self,
        }
    }

    fn with_resource_name_opt(self, name: Option<String>) -> Self {
        match name {
            Some(name) => self.with_resource_name(name),
            None => self,
        }
    }

    fn with_changes_opt(self, changes: Option<HashMap<String, FieldChange>>) -> Self {
        match changes {
            Some(changes) => self.with_changes(changes),
            None => self,
        }
    }

    fn with_actor(
        mut self,
        id: Option<Uuid>,
        actor_type: ActorType,
        email: Option<String>,
    ) -> Self {
        self.actor_id = id;
        self.actor_type = actor_type;
        self.actor_email = email;
        self
    }
}

/// Convenience functions for common audit log patterns.
pub mod audit_helpers {
    use super::*;

    /// Create audit log for organization creation.
    pub fn org_created(org_id: Uuid, user_id: Uuid, org_name: &str) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::OrgCreate)
            .on_resource("organization", org_id.to_string())
            .with_resource_name(org_name)
            .build()
    }

    /// Create audit log for organization update.
    pub fn org_updated(
        org_id: Uuid,
        user_id: Uuid,
        org_name: &str,
        changes: HashMap<String, FieldChange>,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::OrgUpdate)
            .on_resource("organization", org_id.to_string())
            .with_resource_name(org_name)
            .with_changes(changes)
            .build()
    }

    /// Create audit log for device registration.
    pub fn device_registered(
        org_id: Uuid,
        user_id: Option<Uuid>,
        device_id: &str,
        device_name: &str,
    ) -> CreateAuditLogInput {
        let builder = if let Some(uid) = user_id {
            AuditLogBuilder::user_action(org_id, uid, AuditAction::DeviceRegister)
        } else {
            AuditLogBuilder::system_action(org_id, AuditAction::DeviceRegister)
        };
        builder
            .on_resource("device", device_id)
            .with_resource_name(device_name)
            .build()
    }

    /// Create audit log for device enrollment.
    pub fn device_enrolled(
        org_id: Uuid,
        device_id: &str,
        device_name: &str,
        token_id: Option<Uuid>,
    ) -> CreateAuditLogInput {
        let mut builder = AuditLogBuilder::system_action(org_id, AuditAction::DeviceEnroll)
            .on_resource("device", device_id)
            .with_resource_name(device_name);

        if let Some(tid) = token_id {
            builder = builder.with_change("enrollment_token_id", None, Some(tid.to_string()));
        }
        builder.build()
    }

    /// Create audit log for device assignment.
    pub fn device_assigned(
        org_id: Uuid,
        user_id: Uuid,
        device_id: &str,
        device_name: &str,
        assigned_user_id: Uuid,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::DeviceAssign)
            .on_resource("device", device_id)
            .with_resource_name(device_name)
            .with_change("assigned_user_id", None, Some(assigned_user_id.to_string()))
            .build()
    }

    /// Create audit log for device unassignment.
    pub fn device_unassigned(
        org_id: Uuid,
        user_id: Uuid,
        device_id: &str,
        device_name: &str,
        previous_user_id: Uuid,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::DeviceUnassign)
            .on_resource("device", device_id)
            .with_resource_name(device_name)
            .with_change("assigned_user_id", Some(previous_user_id.to_string()), None)
            .build()
    }

    /// Create audit log for policy creation.
    pub fn policy_created(
        org_id: Uuid,
        user_id: Uuid,
        policy_id: Uuid,
        policy_name: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::PolicyCreate)
            .on_resource("policy", policy_id.to_string())
            .with_resource_name(policy_name)
            .build()
    }

    /// Create audit log for policy update.
    pub fn policy_updated(
        org_id: Uuid,
        user_id: Uuid,
        policy_id: Uuid,
        policy_name: &str,
        changes: HashMap<String, FieldChange>,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::PolicyUpdate)
            .on_resource("policy", policy_id.to_string())
            .with_resource_name(policy_name)
            .with_changes(changes)
            .build()
    }

    /// Create audit log for policy deletion.
    pub fn policy_deleted(
        org_id: Uuid,
        user_id: Uuid,
        policy_id: Uuid,
        policy_name: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::PolicyDelete)
            .on_resource("policy", policy_id.to_string())
            .with_resource_name(policy_name)
            .build()
    }

    /// Create audit log for policy application to devices/groups.
    pub fn policy_applied(
        org_id: Uuid,
        user_id: Uuid,
        policy_id: Uuid,
        policy_name: &str,
        target_count: usize,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::PolicyApply)
            .on_resource("policy", policy_id.to_string())
            .with_resource_name(policy_name)
            .with_change("applied_to_count", None, Some(target_count.to_string()))
            .build()
    }

    /// Create audit log for enrollment token creation.
    pub fn token_created(
        org_id: Uuid,
        user_id: Uuid,
        token_id: Uuid,
        token_prefix: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::TokenCreate)
            .on_resource("enrollment_token", token_id.to_string())
            .with_resource_name(format!("Token {}", token_prefix))
            .build()
    }

    /// Create audit log for enrollment token revocation.
    pub fn token_revoked(
        org_id: Uuid,
        user_id: Uuid,
        token_id: Uuid,
        token_prefix: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::TokenRevoke)
            .on_resource("enrollment_token", token_id.to_string())
            .with_resource_name(format!("Token {}", token_prefix))
            .build()
    }

    /// Create audit log for enrollment token usage.
    pub fn token_used(org_id: Uuid, token_id: Uuid, device_id: &str) -> CreateAuditLogInput {
        AuditLogBuilder::system_action(org_id, AuditAction::TokenUse)
            .on_resource("enrollment_token", token_id.to_string())
            .with_change("used_by_device", None, Some(device_id.to_string()))
            .build()
    }

    /// Create audit log for user invite.
    pub fn user_invited(
        org_id: Uuid,
        inviter_id: Uuid,
        invited_email: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, inviter_id, AuditAction::UserInvite)
            .on_resource("user", invited_email)
            .with_resource_name(invited_email)
            .build()
    }

    /// Create audit log for user role change.
    pub fn user_role_changed(
        org_id: Uuid,
        actor_id: Uuid,
        target_user_id: Uuid,
        old_role: &str,
        new_role: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, actor_id, AuditAction::UserRoleChange)
            .on_resource("org_user", target_user_id.to_string())
            .with_change(
                "role",
                Some(old_role.to_string()),
                Some(new_role.to_string()),
            )
            .build()
    }

    /// Create audit log for org user addition.
    pub fn org_user_added(
        org_id: Uuid,
        actor_id: Uuid,
        new_user_id: Uuid,
        email: &str,
        role: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, actor_id, AuditAction::OrgUserAdd)
            .on_resource("org_user", new_user_id.to_string())
            .with_resource_name(email)
            .with_change("role", None, Some(role.to_string()))
            .build()
    }

    /// Create audit log for org user removal.
    pub fn org_user_removed(
        org_id: Uuid,
        actor_id: Uuid,
        removed_user_id: Uuid,
        email: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, actor_id, AuditAction::OrgUserRemove)
            .on_resource("org_user", removed_user_id.to_string())
            .with_resource_name(email)
            .build()
    }

    /// Create audit log for device settings change.
    pub fn device_settings_changed(
        org_id: Uuid,
        user_id: Uuid,
        device_id: &str,
        changes: HashMap<String, FieldChange>,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::DeviceSettingsChange)
            .on_resource("device", device_id)
            .with_changes(changes)
            .build()
    }

    /// Create audit log for settings lock.
    pub fn settings_locked(
        org_id: Uuid,
        user_id: Uuid,
        device_id: &str,
        setting_key: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::SettingsLock)
            .on_resource("device", device_id)
            .with_change("locked_setting", None, Some(setting_key.to_string()))
            .build()
    }

    /// Create audit log for settings unlock.
    pub fn settings_unlocked(
        org_id: Uuid,
        user_id: Uuid,
        device_id: &str,
        setting_key: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::SettingsUnlock)
            .on_resource("device", device_id)
            .with_change("unlocked_setting", Some(setting_key.to_string()), None)
            .build()
    }

    /// Create audit log for group creation.
    pub fn group_created(
        org_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
        group_name: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::GroupCreate)
            .on_resource("group", group_id.to_string())
            .with_resource_name(group_name)
            .build()
    }

    /// Create audit log for group member addition.
    pub fn group_member_added(
        org_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
        group_name: &str,
        member_id: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::GroupMemberAdd)
            .on_resource("group", group_id.to_string())
            .with_resource_name(group_name)
            .with_change("member_added", None, Some(member_id.to_string()))
            .build()
    }

    /// Create audit log for group member removal.
    pub fn group_member_removed(
        org_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
        group_name: &str,
        member_id: &str,
    ) -> CreateAuditLogInput {
        AuditLogBuilder::user_action(org_id, user_id, AuditAction::GroupMemberRemove)
            .on_resource("group", group_id.to_string())
            .with_resource_name(group_name)
            .with_change("member_removed", Some(member_id.to_string()), None)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_action_builder() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let input = AuditLogBuilder::user_action(org_id, user_id, AuditAction::PolicyCreate)
            .on_resource("policy", "policy-123")
            .with_resource_name("Default Policy")
            .with_actor_email("admin@example.com")
            .build();

        assert_eq!(input.organization_id, org_id);
        assert_eq!(input.actor_id, Some(user_id));
        assert_eq!(input.actor_type, ActorType::User);
        assert_eq!(input.resource_type, "policy");
        assert_eq!(input.resource_id, Some("policy-123".to_string()));
    }

    #[test]
    fn test_system_action_builder() {
        let org_id = Uuid::new_v4();

        let input = AuditLogBuilder::system_action(org_id, AuditAction::DeviceEnroll)
            .on_resource("device", "device-456")
            .build();

        assert_eq!(input.organization_id, org_id);
        assert_eq!(input.actor_id, None);
        assert_eq!(input.actor_type, ActorType::System);
    }

    #[test]
    fn test_api_key_action_builder() {
        let org_id = Uuid::new_v4();
        let api_key_id = Uuid::new_v4();

        let input =
            AuditLogBuilder::api_key_action(org_id, api_key_id, AuditAction::DeviceRegister)
                .on_resource("device", "device-789")
                .build();

        assert_eq!(input.organization_id, org_id);
        assert_eq!(input.actor_id, Some(api_key_id));
        assert_eq!(input.actor_type, ActorType::ApiKey);
    }

    #[test]
    fn test_with_changes() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let input = AuditLogBuilder::user_action(org_id, user_id, AuditAction::PolicyUpdate)
            .on_resource("policy", "policy-123")
            .with_change(
                "name",
                Some("Old Name".to_string()),
                Some("New Name".to_string()),
            )
            .with_change("priority", Some("1".to_string()), Some("2".to_string()))
            .build();

        assert!(input.changes.is_some());
        let changes = input.changes.unwrap();
        assert_eq!(changes.len(), 2);
        assert!(changes.contains_key("name"));
        assert!(changes.contains_key("priority"));
    }

    #[test]
    fn test_org_created_helper() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let input = audit_helpers::org_created(org_id, user_id, "Acme Corp");

        assert_eq!(input.organization_id, org_id);
        assert_eq!(input.action, AuditAction::OrgCreate);
        assert_eq!(input.resource_type, "organization");
        assert_eq!(input.resource_name, Some("Acme Corp".to_string()));
    }

    #[test]
    fn test_policy_created_helper() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let policy_id = Uuid::new_v4();

        let input = audit_helpers::policy_created(org_id, user_id, policy_id, "Secure Policy");

        assert_eq!(input.action, AuditAction::PolicyCreate);
        assert_eq!(input.resource_type, "policy");
        assert_eq!(input.resource_id, Some(policy_id.to_string()));
    }

    #[test]
    fn test_token_created_helper() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let token_id = Uuid::new_v4();

        let input = audit_helpers::token_created(org_id, user_id, token_id, "ABC123");

        assert_eq!(input.action, AuditAction::TokenCreate);
        assert_eq!(input.resource_type, "enrollment_token");
        assert_eq!(input.resource_name, Some("Token ABC123".to_string()));
    }

    #[test]
    fn test_device_enrolled_helper() {
        let org_id = Uuid::new_v4();
        let token_id = Uuid::new_v4();

        let input =
            audit_helpers::device_enrolled(org_id, "device-123", "Test Device", Some(token_id));

        assert_eq!(input.action, AuditAction::DeviceEnroll);
        assert_eq!(input.actor_type, ActorType::System);
        assert!(input.changes.is_some());
    }

    #[test]
    fn test_user_role_changed_helper() {
        let org_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();
        let target_user_id = Uuid::new_v4();

        let input =
            audit_helpers::user_role_changed(org_id, actor_id, target_user_id, "member", "admin");

        assert_eq!(input.action, AuditAction::UserRoleChange);
        assert!(input.changes.is_some());
        let changes = input.changes.unwrap();
        assert!(changes.contains_key("role"));
    }

    #[test]
    fn test_group_member_added_helper() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();

        let input = audit_helpers::group_member_added(
            org_id,
            user_id,
            group_id,
            "Engineering",
            "device-123",
        );

        assert_eq!(input.action, AuditAction::GroupMemberAdd);
        assert_eq!(input.resource_type, "group");
        assert!(input.changes.is_some());
    }
}
