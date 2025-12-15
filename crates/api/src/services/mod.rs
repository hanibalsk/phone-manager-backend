//! External service integrations.

pub mod admin_bootstrap;
pub mod apple_auth;
pub mod auth;
pub mod cookies;
pub mod email;
pub mod fcm;
pub mod map_matching;
pub mod path_correction;
pub mod report_generation;
pub mod webhook_delivery;

#[allow(unused_imports)] // Used in routes
pub use apple_auth::AppleAuthClient;
#[allow(unused_imports)] // Used in routes
pub use auth::AuthService;
#[allow(unused_imports)] // Used for httpOnly cookie authentication
pub use cookies::CookieHelper;
#[allow(unused_imports)] // Used for email verification and password reset
pub use email::{EmailError, EmailMessage, EmailService};
#[allow(unused_imports)] // Used when FCM is enabled
pub use fcm::{FcmError, FcmNotificationService};
#[allow(unused_imports)] // Public API for external use
pub use map_matching::{MapMatchingClient, MapMatchingResult};
pub use path_correction::PathCorrectionService;
#[allow(unused_imports)] // Used in report generation job
pub use report_generation::{ReportGenerationError, ReportGenerationService};
#[allow(unused_imports)] // Used in geofence_events routes
pub use webhook_delivery::WebhookDeliveryService;
