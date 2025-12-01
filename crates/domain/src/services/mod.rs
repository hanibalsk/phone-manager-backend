//! Domain services for Phone Manager.
//!
//! Services contain business logic that operates on domain models.

pub mod notification;

pub use notification::{
    MockNotificationService, NotificationPayload, NotificationResult, NotificationService,
    NotificationType, SettingChangeAction, SettingChangeNotification, SettingsChangedPayload,
    UnlockRequestResponsePayload,
};
