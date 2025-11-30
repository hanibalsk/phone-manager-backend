//! Repository implementations for database operations.

pub mod api_key;
pub mod device;
pub mod geofence;
pub mod idempotency_key;
pub mod location;
pub mod movement_event;
pub mod proximity_alert;

pub use api_key::ApiKeyRepository;
pub use device::{AdminStats, DeviceRepository};
pub use geofence::GeofenceRepository;
pub use idempotency_key::IdempotencyKeyRepository;
pub use location::{LocationHistoryQuery, LocationInput, LocationRepository};
pub use movement_event::{MovementEventInput, MovementEventQuery, MovementEventRepository};
pub use proximity_alert::ProximityAlertRepository;
