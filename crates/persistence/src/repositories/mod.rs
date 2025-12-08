//! Repository implementations for database operations.

pub mod admin_group;
pub mod admin_user;
pub mod api_key;
pub mod audit_export_job;
pub mod audit_log;
pub mod dashboard;
pub mod device;
pub mod device_command;
pub mod device_policy;
pub mod device_token;
pub mod enrollment_token;
pub mod geofence;
pub mod geofence_event;
pub mod group;
pub mod idempotency_key;
pub mod invite;
pub mod location;
pub mod movement_event;
pub mod org_user;
pub mod organization;
pub mod proximity_alert;
pub mod registration_invite;
pub mod setting;
pub mod trip;
pub mod trip_path_correction;
pub mod unlock_request;
pub mod user;
pub mod webhook;
pub mod webhook_delivery;

pub use admin_group::AdminGroupRepository;
pub use admin_user::AdminUserRepository;
pub use api_key::ApiKeyRepository;
pub use audit_export_job::{AuditExportJobRepository, ExportJob};
pub use audit_log::AuditLogRepository;
pub use dashboard::DashboardRepository;
pub use device::{AdminStats, DeviceRepository, FleetSummaryCounts};
pub use device_command::DeviceCommandRepository;
pub use device_policy::DevicePolicyRepository;
pub use device_token::DeviceTokenRepository;
pub use enrollment_token::EnrollmentTokenRepository;
pub use geofence::GeofenceRepository;
pub use geofence_event::GeofenceEventRepository;
pub use group::GroupRepository;
pub use idempotency_key::IdempotencyKeyRepository;
pub use invite::InviteRepository;
pub use location::{LocationHistoryQuery, LocationInput, LocationRepository};
pub use movement_event::{MovementEventInput, MovementEventQuery, MovementEventRepository};
pub use org_user::OrgUserRepository;
pub use organization::OrganizationRepository;
pub use proximity_alert::ProximityAlertRepository;
pub use registration_invite::{
    default_expiration, generate_invite_token, RegistrationInviteRepository,
};
pub use setting::SettingRepository;
pub use trip::{TripInput, TripQuery, TripRepository, TripUpdateInput};
pub use trip_path_correction::{
    TripPathCorrectionInput, TripPathCorrectionRepository, TripPathCorrectionUpdateInput,
};
pub use unlock_request::UnlockRequestRepository;
pub use user::UserRepository;
pub use webhook::WebhookRepository;
pub use webhook_delivery::{DeliveryStats, WebhookDeliveryRepository};
