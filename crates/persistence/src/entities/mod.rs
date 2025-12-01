//! Database entity definitions.
//!
//! Entities are direct mappings to database rows.

pub mod api_key;
pub mod device;
pub mod geofence;
pub mod group;
pub mod idempotency_key;
pub mod location;
pub mod movement_event;
pub mod proximity_alert;
pub mod trip;
pub mod trip_path_correction;
pub mod user;

pub use api_key::ApiKeyEntity;
pub use device::{DeviceEntity, DeviceWithLastLocationEntity};
pub use geofence::GeofenceEntity;
pub use group::{
    GroupEntity, GroupMembershipEntity, GroupRoleDb, GroupWithMembershipEntity, MemberWithUserEntity,
};
pub use idempotency_key::IdempotencyKeyEntity;
pub use location::LocationEntity;
pub use movement_event::MovementEventEntity;
pub use proximity_alert::ProximityAlertEntity;
pub use trip::TripEntity;
pub use trip_path_correction::TripPathCorrectionEntity;
pub use user::{OAuthAccountEntity, UserEntity, UserSessionEntity};
