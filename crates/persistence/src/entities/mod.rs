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
pub mod group;
pub mod idempotency_key;
pub mod invite;
pub mod location;
pub mod movement_event;
pub mod org_user;
pub mod organization;
pub mod proximity_alert;
pub mod setting;
pub mod trip;
pub mod trip_path_correction;
pub mod unlock_request;
pub mod user;

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
pub use group::{
    GroupEntity, GroupMembershipEntity, GroupRoleDb, GroupWithMembershipEntity, MemberWithUserEntity,
};
pub use idempotency_key::IdempotencyKeyEntity;
pub use invite::{GroupInviteEntity, InviteWithCreatorEntity, InviteWithGroupEntity};
pub use location::LocationEntity;
pub use movement_event::MovementEventEntity;
pub use org_user::{OrgUserEntity, OrgUserRoleDb, OrgUserWithDetailsEntity};
pub use organization::{OrganizationEntity, OrganizationWithUsageEntity, PlanTypeDb};
pub use proximity_alert::ProximityAlertEntity;
pub use setting::{
    DeviceSettingEntity, DeviceSettingWithDefinitionEntity, SettingCategoryDb, SettingDataTypeDb,
    SettingDefinitionEntity, SettingLockEntity,
};
pub use trip::TripEntity;
pub use trip_path_correction::TripPathCorrectionEntity;
pub use unlock_request::{UnlockRequestEntity, UnlockRequestStatusDb, UnlockRequestWithDetailsEntity};
pub use user::{OAuthAccountEntity, UserEntity, UserSessionEntity};
