//! Domain models for Phone Manager.

pub mod device;
pub mod geofence;
pub mod group;
pub mod invite;
pub mod location;
pub mod movement_event;
pub mod proximity_alert;
pub mod setting;
pub mod trip;
pub mod trip_path_correction;
pub mod unlock_request;
pub mod user;

pub use device::Device;
pub use geofence::Geofence;
pub use group::{Group, GroupMembership, GroupRole};
pub use invite::GroupInvite;
pub use location::Location;
pub use movement_event::MovementEvent;
pub use proximity_alert::ProximityAlert;
pub use setting::{
    DeviceSetting, GetSettingsResponse, SettingCategory, SettingDataType, SettingDefinition,
    SettingValue,
};
pub use trip::Trip;
pub use trip_path_correction::TripPathCorrection;
pub use unlock_request::{
    CreateUnlockRequestRequest, CreateUnlockRequestResponse, ListUnlockRequestsQuery,
    ListUnlockRequestsResponse, RespondToUnlockRequestRequest, RespondToUnlockRequestResponse,
    UnlockRequestStatus,
};
pub use user::{OAuthAccount, OAuthProvider, User, UserSession};
