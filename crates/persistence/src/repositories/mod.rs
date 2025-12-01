//! Repository implementations for database operations.

pub mod api_key;
pub mod device;
pub mod geofence;
pub mod group;
pub mod idempotency_key;
pub mod invite;
pub mod location;
pub mod movement_event;
pub mod proximity_alert;
pub mod setting;
pub mod trip;
pub mod trip_path_correction;
pub mod unlock_request;
pub mod user;

pub use api_key::ApiKeyRepository;
pub use device::{AdminStats, DeviceRepository};
pub use geofence::GeofenceRepository;
pub use group::GroupRepository;
pub use idempotency_key::IdempotencyKeyRepository;
pub use invite::InviteRepository;
pub use location::{LocationHistoryQuery, LocationInput, LocationRepository};
pub use movement_event::{MovementEventInput, MovementEventQuery, MovementEventRepository};
pub use proximity_alert::ProximityAlertRepository;
pub use setting::SettingRepository;
pub use trip::{TripInput, TripQuery, TripRepository, TripUpdateInput};
pub use trip_path_correction::{
    TripPathCorrectionInput, TripPathCorrectionRepository, TripPathCorrectionUpdateInput,
};
pub use unlock_request::UnlockRequestRepository;
pub use user::UserRepository;
