//! Repository implementations for database operations.

pub mod admin_geofence;
pub mod admin_group;
pub mod admin_user;
pub mod analytics;
pub mod api_key;
pub mod app_usage;
pub mod audit_export_job;
pub mod audit_log;
pub mod dashboard;
pub mod data_subject_request;
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
pub mod managed_user;
pub mod migration_audit;
pub mod movement_event;
pub mod org_member_invite;
pub mod org_user;
pub mod org_webhook;
pub mod organization;
pub mod organization_role;
pub mod organization_settings;
pub mod proximity_alert;
pub mod registration_invite;
pub mod setting;
pub mod setting_change;
pub mod system_config;
pub mod system_role;
pub mod trip;
pub mod trip_path_correction;
pub mod unlock_request;
pub mod user;
pub mod user_geofence;
pub mod webhook;
pub mod webhook_delivery;

pub use admin_geofence::AdminGeofenceRepository;
pub use admin_group::AdminGroupRepository;
pub use admin_user::AdminUserRepository;
pub use analytics::AnalyticsRepository;
pub use api_key::ApiKeyRepository;
pub use app_usage::AppUsageRepository;
pub use audit_export_job::{AuditExportJobRepository, ExportJob};
pub use audit_log::AuditLogRepository;
pub use dashboard::DashboardRepository;
pub use data_subject_request::{
    CreateDataSubjectRequestInput, DataSubjectRequestCounts, DataSubjectRequestRepository,
    ListDataSubjectRequestsQuery, ProcessDataSubjectRequestInput, DEFAULT_DUE_DAYS,
};
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
pub use managed_user::ManagedUserRepository;
pub use migration_audit::{
    CreateMigrationAuditInput, ListMigrationAuditQuery, MigrationAuditRepository,
};
pub use movement_event::{MovementEventInput, MovementEventQuery, MovementEventRepository};
pub use org_member_invite::{
    calculate_invite_expiration, default_invite_expiration, generate_org_member_invite_token,
    InviteSummaryCounts, OrgMemberInviteRepository,
};
pub use org_user::OrgUserRepository;
pub use org_webhook::OrgWebhookRepository;
pub use organization::OrganizationRepository;
pub use organization_role::OrganizationRoleRepository;
pub use organization_settings::OrganizationSettingsRepository;
pub use proximity_alert::ProximityAlertRepository;
pub use registration_invite::{
    default_expiration, generate_invite_token, RegistrationInviteRepository,
};
pub use setting::SettingRepository;
pub use setting_change::{CreateSettingChangeInput, SettingChangeRepository};
pub use system_config::SystemConfigRepository;
pub use system_role::SystemRoleRepository;
pub use trip::{TripInput, TripQuery, TripRepository, TripUpdateInput};
pub use trip_path_correction::{
    TripPathCorrectionInput, TripPathCorrectionRepository, TripPathCorrectionUpdateInput,
};
pub use unlock_request::UnlockRequestRepository;
pub use user::{MfaStatusRow, UserRepository, UserSessionRow};
pub use user_geofence::UserGeofenceRepository;
pub use webhook::WebhookRepository;
pub use webhook_delivery::{DeliveryStats, WebhookDeliveryRepository, WebhookDeliveryStats};
