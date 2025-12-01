//! Audit log domain models.
//!
//! Story 13.9: Audit Logging System

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use uuid::Uuid;

/// Actor types that can perform audited actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    /// Human user.
    User,
    /// Automated system process.
    System,
    /// External API key integration.
    ApiKey,
}

impl FromStr for ActorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(ActorType::User),
            "system" => Ok(ActorType::System),
            "api_key" => Ok(ActorType::ApiKey),
            _ => Err(format!("Unknown actor type: {}", s)),
        }
    }
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorType::User => write!(f, "user"),
            ActorType::System => write!(f, "system"),
            ActorType::ApiKey => write!(f, "api_key"),
        }
    }
}

/// Resource types that can be audited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Organization,
    User,
    Device,
    Policy,
    Group,
    Settings,
    Token,
    EnrollmentToken,
    OrgUser,
}

impl FromStr for ResourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "organization" | "org" => Ok(ResourceType::Organization),
            "user" => Ok(ResourceType::User),
            "device" => Ok(ResourceType::Device),
            "policy" => Ok(ResourceType::Policy),
            "group" => Ok(ResourceType::Group),
            "settings" => Ok(ResourceType::Settings),
            "token" => Ok(ResourceType::Token),
            "enrollment_token" => Ok(ResourceType::EnrollmentToken),
            "org_user" => Ok(ResourceType::OrgUser),
            _ => Err(format!("Unknown resource type: {}", s)),
        }
    }
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::Organization => write!(f, "organization"),
            ResourceType::User => write!(f, "user"),
            ResourceType::Device => write!(f, "device"),
            ResourceType::Policy => write!(f, "policy"),
            ResourceType::Group => write!(f, "group"),
            ResourceType::Settings => write!(f, "settings"),
            ResourceType::Token => write!(f, "token"),
            ResourceType::EnrollmentToken => write!(f, "enrollment_token"),
            ResourceType::OrgUser => write!(f, "org_user"),
        }
    }
}

/// Audited actions following the format: resource.operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Organization actions
    OrgCreate,
    OrgUpdate,
    OrgDelete,
    OrgSettingsChange,

    // User actions
    UserCreate,
    UserUpdate,
    UserDelete,
    UserRoleChange,
    UserInvite,

    // Device actions
    DeviceRegister,
    DeviceEnroll,
    DeviceAssign,
    DeviceUnassign,
    DeviceSuspend,
    DeviceRetire,
    DeviceWipe,
    DeviceSettingsChange,

    // Policy actions
    PolicyCreate,
    PolicyUpdate,
    PolicyDelete,
    PolicyApply,
    PolicyUnapply,

    // Group actions
    GroupCreate,
    GroupUpdate,
    GroupDelete,
    GroupMemberAdd,
    GroupMemberRemove,

    // Settings actions
    SettingsLock,
    SettingsUnlock,
    SettingsOverride,
    SettingsBulkUpdate,

    // Token actions
    TokenCreate,
    TokenRevoke,
    TokenUse,

    // Org user actions
    OrgUserAdd,
    OrgUserUpdate,
    OrgUserRemove,
}

impl FromStr for AuditAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "org.create" => Ok(AuditAction::OrgCreate),
            "org.update" => Ok(AuditAction::OrgUpdate),
            "org.delete" => Ok(AuditAction::OrgDelete),
            "org.settings_change" => Ok(AuditAction::OrgSettingsChange),
            "user.create" => Ok(AuditAction::UserCreate),
            "user.update" => Ok(AuditAction::UserUpdate),
            "user.delete" => Ok(AuditAction::UserDelete),
            "user.role_change" => Ok(AuditAction::UserRoleChange),
            "user.invite" => Ok(AuditAction::UserInvite),
            "device.register" => Ok(AuditAction::DeviceRegister),
            "device.enroll" => Ok(AuditAction::DeviceEnroll),
            "device.assign" => Ok(AuditAction::DeviceAssign),
            "device.unassign" => Ok(AuditAction::DeviceUnassign),
            "device.suspend" => Ok(AuditAction::DeviceSuspend),
            "device.retire" => Ok(AuditAction::DeviceRetire),
            "device.wipe" => Ok(AuditAction::DeviceWipe),
            "device.settings_change" => Ok(AuditAction::DeviceSettingsChange),
            "policy.create" => Ok(AuditAction::PolicyCreate),
            "policy.update" => Ok(AuditAction::PolicyUpdate),
            "policy.delete" => Ok(AuditAction::PolicyDelete),
            "policy.apply" => Ok(AuditAction::PolicyApply),
            "policy.unapply" => Ok(AuditAction::PolicyUnapply),
            "group.create" => Ok(AuditAction::GroupCreate),
            "group.update" => Ok(AuditAction::GroupUpdate),
            "group.delete" => Ok(AuditAction::GroupDelete),
            "group.member_add" => Ok(AuditAction::GroupMemberAdd),
            "group.member_remove" => Ok(AuditAction::GroupMemberRemove),
            "settings.lock" => Ok(AuditAction::SettingsLock),
            "settings.unlock" => Ok(AuditAction::SettingsUnlock),
            "settings.override" => Ok(AuditAction::SettingsOverride),
            "settings.bulk_update" => Ok(AuditAction::SettingsBulkUpdate),
            "token.create" => Ok(AuditAction::TokenCreate),
            "token.revoke" => Ok(AuditAction::TokenRevoke),
            "token.use" => Ok(AuditAction::TokenUse),
            "org_user.add" => Ok(AuditAction::OrgUserAdd),
            "org_user.update" => Ok(AuditAction::OrgUserUpdate),
            "org_user.remove" => Ok(AuditAction::OrgUserRemove),
            _ => Err(format!("Unknown audit action: {}", s)),
        }
    }
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AuditAction::OrgCreate => "org.create",
            AuditAction::OrgUpdate => "org.update",
            AuditAction::OrgDelete => "org.delete",
            AuditAction::OrgSettingsChange => "org.settings_change",
            AuditAction::UserCreate => "user.create",
            AuditAction::UserUpdate => "user.update",
            AuditAction::UserDelete => "user.delete",
            AuditAction::UserRoleChange => "user.role_change",
            AuditAction::UserInvite => "user.invite",
            AuditAction::DeviceRegister => "device.register",
            AuditAction::DeviceEnroll => "device.enroll",
            AuditAction::DeviceAssign => "device.assign",
            AuditAction::DeviceUnassign => "device.unassign",
            AuditAction::DeviceSuspend => "device.suspend",
            AuditAction::DeviceRetire => "device.retire",
            AuditAction::DeviceWipe => "device.wipe",
            AuditAction::DeviceSettingsChange => "device.settings_change",
            AuditAction::PolicyCreate => "policy.create",
            AuditAction::PolicyUpdate => "policy.update",
            AuditAction::PolicyDelete => "policy.delete",
            AuditAction::PolicyApply => "policy.apply",
            AuditAction::PolicyUnapply => "policy.unapply",
            AuditAction::GroupCreate => "group.create",
            AuditAction::GroupUpdate => "group.update",
            AuditAction::GroupDelete => "group.delete",
            AuditAction::GroupMemberAdd => "group.member_add",
            AuditAction::GroupMemberRemove => "group.member_remove",
            AuditAction::SettingsLock => "settings.lock",
            AuditAction::SettingsUnlock => "settings.unlock",
            AuditAction::SettingsOverride => "settings.override",
            AuditAction::SettingsBulkUpdate => "settings.bulk_update",
            AuditAction::TokenCreate => "token.create",
            AuditAction::TokenRevoke => "token.revoke",
            AuditAction::TokenUse => "token.use",
            AuditAction::OrgUserAdd => "org_user.add",
            AuditAction::OrgUserUpdate => "org_user.update",
            AuditAction::OrgUserRemove => "org_user.remove",
        };
        write!(f, "{}", s)
    }
}

/// Represents a change to a field with old and new values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub old: Option<JsonValue>,
    pub new: Option<JsonValue>,
}

impl FieldChange {
    /// Create a new field change.
    pub fn new(old: Option<JsonValue>, new: Option<JsonValue>) -> Self {
        Self { old, new }
    }
}

/// Audit log entry domain model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor: AuditActor,
    pub action: String,
    pub resource: AuditResource,
    pub changes: Option<HashMap<String, FieldChange>>,
    pub metadata: Option<AuditMetadata>,
}

/// Actor information for audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditActor {
    pub id: Option<Uuid>,
    #[serde(rename = "type")]
    pub actor_type: ActorType,
    pub email: Option<String>,
}

impl AuditActor {
    /// Create a user actor.
    pub fn user(id: Uuid, email: Option<String>) -> Self {
        Self {
            id: Some(id),
            actor_type: ActorType::User,
            email,
        }
    }

    /// Create a system actor.
    pub fn system() -> Self {
        Self {
            id: None,
            actor_type: ActorType::System,
            email: None,
        }
    }

    /// Create an API key actor.
    pub fn api_key(id: Uuid) -> Self {
        Self {
            id: Some(id),
            actor_type: ActorType::ApiKey,
            email: None,
        }
    }
}

/// Resource information for audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditResource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: Option<String>,
    pub name: Option<String>,
}

impl AuditResource {
    /// Create a new audit resource.
    pub fn new(resource_type: impl Into<String>, id: Option<impl Into<String>>, name: Option<impl Into<String>>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: id.map(Into::into),
            name: name.map(Into::into),
        }
    }
}

/// Metadata for audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditMetadata {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    #[serde(flatten)]
    pub extra: Option<JsonValue>,
}

impl Default for AuditMetadata {
    fn default() -> Self {
        Self {
            ip_address: None,
            user_agent: None,
            request_id: None,
            extra: None,
        }
    }
}

impl AuditMetadata {
    /// Create new metadata with common fields.
    pub fn new(ip_address: Option<IpAddr>, user_agent: Option<String>, request_id: Option<String>) -> Self {
        Self {
            ip_address: ip_address.map(|ip| ip.to_string()),
            user_agent,
            request_id,
            extra: None,
        }
    }

    /// Add extra metadata.
    pub fn with_extra(mut self, extra: JsonValue) -> Self {
        self.extra = Some(extra);
        self
    }
}

/// Input for creating a new audit log entry.
#[derive(Debug, Clone)]
pub struct CreateAuditLogInput {
    pub organization_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_type: ActorType,
    pub actor_email: Option<String>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub changes: Option<HashMap<String, FieldChange>>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

impl CreateAuditLogInput {
    /// Create a new audit log input builder.
    pub fn new(organization_id: Uuid, action: AuditAction, resource_type: impl Into<String>) -> Self {
        Self {
            organization_id,
            actor_id: None,
            actor_type: ActorType::System,
            actor_email: None,
            action,
            resource_type: resource_type.into(),
            resource_id: None,
            resource_name: None,
            changes: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
        }
    }

    /// Set the actor as a user.
    pub fn with_user_actor(mut self, user_id: Uuid, email: Option<String>) -> Self {
        self.actor_id = Some(user_id);
        self.actor_type = ActorType::User;
        self.actor_email = email;
        self
    }

    /// Set the actor as a system.
    pub fn with_system_actor(mut self) -> Self {
        self.actor_id = None;
        self.actor_type = ActorType::System;
        self.actor_email = None;
        self
    }

    /// Set the actor as an API key.
    pub fn with_api_key_actor(mut self, api_key_id: Uuid) -> Self {
        self.actor_id = Some(api_key_id);
        self.actor_type = ActorType::ApiKey;
        self.actor_email = None;
        self
    }

    /// Set the resource ID.
    pub fn with_resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// Set the resource name.
    pub fn with_resource_name(mut self, name: impl Into<String>) -> Self {
        self.resource_name = Some(name.into());
        self
    }

    /// Set the changes.
    pub fn with_changes(mut self, changes: HashMap<String, FieldChange>) -> Self {
        self.changes = Some(changes);
        self
    }

    /// Add a single field change.
    pub fn add_change(mut self, field: impl Into<String>, old: Option<JsonValue>, new: Option<JsonValue>) -> Self {
        let changes = self.changes.get_or_insert_with(HashMap::new);
        changes.insert(field.into(), FieldChange::new(old, new));
        self
    }

    /// Set request context.
    pub fn with_request_context(mut self, ip_address: Option<IpAddr>, user_agent: Option<String>, request_id: Option<String>) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self.request_id = request_id;
        self
    }
}

/// Query parameters for listing audit logs.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub actor_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

/// Pagination info for audit log list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogPagination {
    pub page: i32,
    pub per_page: i32,
    pub total: i64,
    pub total_pages: i32,
}

/// Response for audit log list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsResponse {
    pub data: Vec<AuditLog>,
    pub pagination: AuditLogPagination,
}

/// Export format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
}

impl FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            _ => Err(format!("Unknown export format: {}", s)),
        }
    }
}

/// Query parameters for exporting audit logs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExportAuditLogsQuery {
    pub format: Option<ExportFormat>,
    pub actor_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl ExportAuditLogsQuery {
    /// Convert to list query for reuse.
    pub fn to_list_query(&self) -> ListAuditLogsQuery {
        ListAuditLogsQuery {
            page: None,
            per_page: None,
            actor_id: self.actor_id,
            action: self.action.clone(),
            resource_type: self.resource_type.clone(),
            resource_id: self.resource_id.clone(),
            from: self.from,
            to: self.to,
        }
    }
}

/// Export job status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Expired,
}

impl std::fmt::Display for ExportJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportJobStatus::Pending => write!(f, "pending"),
            ExportJobStatus::Processing => write!(f, "processing"),
            ExportJobStatus::Completed => write!(f, "completed"),
            ExportJobStatus::Failed => write!(f, "failed"),
            ExportJobStatus::Expired => write!(f, "expired"),
        }
    }
}

impl FromStr for ExportJobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ExportJobStatus::Pending),
            "processing" => Ok(ExportJobStatus::Processing),
            "completed" => Ok(ExportJobStatus::Completed),
            "failed" => Ok(ExportJobStatus::Failed),
            "expired" => Ok(ExportJobStatus::Expired),
            _ => Err(format!("Unknown export job status: {}", s)),
        }
    }
}

/// Sync export response (for small datasets).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncExportResponse {
    pub format: ExportFormat,
    pub record_count: i64,
    pub download_url: String,
}

/// Async export response (for large datasets).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncExportResponse {
    pub job_id: String,
    pub status: ExportJobStatus,
    pub estimated_records: i64,
    pub check_url: String,
}

/// Export job status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportJobResponse {
    pub job_id: String,
    pub status: ExportJobStatus,
    pub record_count: Option<i64>,
    pub download_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// Maximum records for sync export.
pub const MAX_SYNC_EXPORT_RECORDS: i64 = 1000;

/// Maximum records for any export.
pub const MAX_EXPORT_RECORDS: i64 = 10000;

/// Export job expiry in hours.
pub const EXPORT_JOB_EXPIRY_HOURS: i64 = 24;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_type_from_str() {
        assert_eq!(ActorType::from_str("user").unwrap(), ActorType::User);
        assert_eq!(ActorType::from_str("system").unwrap(), ActorType::System);
        assert_eq!(ActorType::from_str("api_key").unwrap(), ActorType::ApiKey);
        assert!(ActorType::from_str("invalid").is_err());
    }

    #[test]
    fn test_actor_type_display() {
        assert_eq!(ActorType::User.to_string(), "user");
        assert_eq!(ActorType::System.to_string(), "system");
        assert_eq!(ActorType::ApiKey.to_string(), "api_key");
    }

    #[test]
    fn test_resource_type_from_str() {
        assert_eq!(ResourceType::from_str("device").unwrap(), ResourceType::Device);
        assert_eq!(ResourceType::from_str("organization").unwrap(), ResourceType::Organization);
        assert!(ResourceType::from_str("invalid").is_err());
    }

    #[test]
    fn test_audit_action_from_str() {
        assert_eq!(AuditAction::from_str("device.assign").unwrap(), AuditAction::DeviceAssign);
        assert_eq!(AuditAction::from_str("org.create").unwrap(), AuditAction::OrgCreate);
        assert!(AuditAction::from_str("invalid.action").is_err());
    }

    #[test]
    fn test_audit_action_display() {
        assert_eq!(AuditAction::DeviceAssign.to_string(), "device.assign");
        assert_eq!(AuditAction::OrgCreate.to_string(), "org.create");
    }

    #[test]
    fn test_create_audit_log_input_builder() {
        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let input = CreateAuditLogInput::new(org_id, AuditAction::DeviceAssign, "device")
            .with_user_actor(user_id, Some("admin@example.com".to_string()))
            .with_resource_id("device-123")
            .with_resource_name("Field Tablet 1")
            .add_change("assigned_user_id", None, Some(serde_json::json!("user-456")));

        assert_eq!(input.organization_id, org_id);
        assert_eq!(input.actor_id, Some(user_id));
        assert_eq!(input.actor_type, ActorType::User);
        assert_eq!(input.action, AuditAction::DeviceAssign);
        assert!(input.changes.is_some());
    }

    #[test]
    fn test_audit_actor_constructors() {
        let user_id = Uuid::new_v4();

        let user_actor = AuditActor::user(user_id, Some("test@example.com".to_string()));
        assert_eq!(user_actor.id, Some(user_id));
        assert_eq!(user_actor.actor_type, ActorType::User);

        let system_actor = AuditActor::system();
        assert!(system_actor.id.is_none());
        assert_eq!(system_actor.actor_type, ActorType::System);

        let api_key_actor = AuditActor::api_key(user_id);
        assert_eq!(api_key_actor.id, Some(user_id));
        assert_eq!(api_key_actor.actor_type, ActorType::ApiKey);
    }

    #[test]
    fn test_audit_metadata() {
        use std::net::Ipv4Addr;

        let metadata = AuditMetadata::new(
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            Some("Mozilla/5.0".to_string()),
            Some("req-123".to_string()),
        );

        assert_eq!(metadata.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(metadata.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(metadata.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_field_change() {
        let change = FieldChange::new(
            Some(serde_json::json!("old_value")),
            Some(serde_json::json!("new_value")),
        );

        assert_eq!(change.old, Some(serde_json::json!("old_value")));
        assert_eq!(change.new, Some(serde_json::json!("new_value")));
    }
}
