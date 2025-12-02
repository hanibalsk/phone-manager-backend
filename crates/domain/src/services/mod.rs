//! Domain services for Phone Manager.
//!
//! Services contain business logic that operates on domain models.

pub mod audit;
pub mod notification;
pub mod policy_resolution;

pub use notification::{
    MockNotificationService, NotificationPayload, NotificationResult, NotificationService,
    NotificationType, SettingChangeAction, SettingChangeNotification, SettingsChangedPayload,
    UnlockRequestResponsePayload,
};

pub use policy_resolution::{
    resolve_effective_settings, needs_resolution, PolicyResolutionInput, PolicySettings,
    ResolvedSettings, SettingSource,
};

pub use audit::{AuditLogBuilder, audit_helpers};
