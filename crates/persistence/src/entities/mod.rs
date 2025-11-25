//! Database entity definitions.
//!
//! Entities are direct mappings to database rows.

pub mod device;
pub mod location;

pub use device::DeviceEntity;
pub use location::LocationEntity;
