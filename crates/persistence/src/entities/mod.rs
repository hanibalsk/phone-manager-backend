//! Database entity definitions.
//!
//! Entities are direct mappings to database rows.

pub mod api_key;
pub mod device;
pub mod geofence;
pub mod idempotency_key;
pub mod location;
pub mod proximity_alert;

pub use api_key::ApiKeyEntity;
pub use device::{DeviceEntity, DeviceWithLastLocationEntity};
pub use geofence::GeofenceEntity;
pub use idempotency_key::IdempotencyKeyEntity;
pub use location::LocationEntity;
pub use proximity_alert::ProximityAlertEntity;
