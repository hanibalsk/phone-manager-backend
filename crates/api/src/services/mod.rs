//! External service integrations.

pub mod auth;
pub mod map_matching;
pub mod path_correction;

#[allow(unused_imports)] // Used in routes
pub use auth::AuthService;
#[allow(unused_imports)] // Public API for external use
pub use map_matching::{MapMatchingClient, MapMatchingResult};
pub use path_correction::PathCorrectionService;
