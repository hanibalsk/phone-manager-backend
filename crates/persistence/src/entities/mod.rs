//! Database entity definitions.
//!
//! Entities are direct mappings to database rows.

pub mod api_key;
pub mod device;
pub mod device_policy;
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

pub use api_key::ApiKeyEntity;
pub use device::{DeviceEntity, DeviceWithLastLocationEntity};
pub use device_policy::DevicePolicyEntity;
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
