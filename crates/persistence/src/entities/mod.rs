//! Database entity definitions.
//!
//! Entities are direct mappings to database rows.

pub mod api_key;
pub mod device;
pub mod geofence;
pub mod idempotency_key;
pub mod location;
pub mod movement_event;
pub mod proximity_alert;
pub mod trip;

pub use api_key::ApiKeyEntity;
pub use device::{DeviceEntity, DeviceWithLastLocationEntity};
pub use geofence::GeofenceEntity;
pub use idempotency_key::IdempotencyKeyEntity;
pub use location::LocationEntity;
pub use movement_event::MovementEventEntity;
pub use proximity_alert::ProximityAlertEntity;
pub use trip::TripEntity;
