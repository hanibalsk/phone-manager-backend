//! Database entity definitions.
//!
//! Entities are direct mappings to database rows.

pub mod api_key;
pub mod device;
pub mod idempotency_key;
pub mod location;

pub use api_key::ApiKeyEntity;
pub use device::{DeviceEntity, DeviceWithLastLocationEntity};
pub use idempotency_key::IdempotencyKeyEntity;
pub use location::LocationEntity;
