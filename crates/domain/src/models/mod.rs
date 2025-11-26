//! Domain models for Phone Manager.

pub mod device;
pub mod geofence;
pub mod location;
pub mod proximity_alert;

pub use device::Device;
pub use geofence::Geofence;
pub use location::Location;
pub use proximity_alert::ProximityAlert;
