//! Database entity definitions.
//!
//! Entities are direct mappings to database rows.

pub mod admin_group;
pub mod admin_user;
pub mod api_key;
pub mod audit_export_job;
pub mod audit_log;
pub mod device;
pub mod device_command;
pub mod device_policy;
pub mod device_token;
pub mod enrollment_token;
pub mod geofence;
pub mod geofence_event;
pub mod group;
pub mod idempotency_key;
pub mod invite;
pub mod location;
pub mod movement_event;
pub mod org_member_invite;
pub mod org_user;
pub mod org_webhook;
pub mod organization;
pub mod organization_settings;
pub mod proximity_alert;
pub mod registration_invite;
pub mod setting;
pub mod system_role;
pub mod trip;
pub mod trip_path_correction;
pub mod unlock_request;
pub mod user;
pub mod webhook;
pub mod webhook_delivery;

pub use admin_group::{
    AdminGroupEntity, AdminGroupProfileEntity, AdminGroupSummaryEntity, GroupDeviceEntity,
    GroupMemberEntity,
};
pub use admin_user::{
    AdminUserEntity, AdminUserProfileEntity, AdminUserSummaryEntity, RecentActionEntity,
    UserDeviceEntity, UserGroupEntity,
};
pub use api_key::ApiKeyEntity;
pub use audit_export_job::AuditExportJobEntity;
pub use audit_log::AuditLogEntity;
pub use device::{DeviceEntity, DeviceWithLastLocationEntity, FleetDeviceEntity};
pub use device_command::DeviceCommandEntity;
pub use device_policy::DevicePolicyEntity;
pub use device_token::DeviceTokenEntity;
pub use enrollment_token::EnrollmentTokenEntity;
pub use geofence::GeofenceEntity;
pub use geofence_event::{GeofenceEventEntity, GeofenceEventWithName};
pub use group::{
    GroupEntity, GroupMembershipEntity, GroupRoleDb, GroupWithMembershipEntity, MemberWithUserEntity,
};
pub use idempotency_key::IdempotencyKeyEntity;
pub use invite::{GroupInviteEntity, InviteWithCreatorEntity, InviteWithGroupEntity};
pub use location::LocationEntity;
pub use movement_event::MovementEventEntity;
pub use org_member_invite::OrgMemberInviteEntity;
pub use org_user::{OrgUserEntity, OrgUserRoleDb, OrgUserWithDetailsEntity};
pub use org_webhook::OrgWebhookEntity;
pub use organization::{OrganizationEntity, OrganizationWithUsageEntity, PlanTypeDb};
pub use organization_settings::OrganizationSettingsEntity;
pub use proximity_alert::ProximityAlertEntity;
pub use registration_invite::RegistrationInviteEntity;
pub use setting::{
    DeviceSettingEntity, DeviceSettingWithDefinitionEntity, SettingCategoryDb, SettingDataTypeDb,
    SettingDefinitionEntity, SettingLockEntity,
};
pub use system_role::{
    AdminOrgAssignmentEntity, AdminOrgAssignmentWithNameEntity, SystemRoleDb, UserSystemRoleEntity,
};
pub use trip::TripEntity;
pub use trip_path_correction::TripPathCorrectionEntity;
pub use unlock_request::{UnlockRequestEntity, UnlockRequestStatusDb, UnlockRequestWithDetailsEntity};
pub use user::{OAuthAccountEntity, UserEntity, UserSessionEntity};
pub use webhook::WebhookEntity;
pub use webhook_delivery::{
    WebhookDeliveryEntity, MAX_RETRY_ATTEMPTS, RETRY_BACKOFF_SECONDS, STATUS_FAILED, STATUS_PENDING,
    STATUS_SUCCESS,
};
