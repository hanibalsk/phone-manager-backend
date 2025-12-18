use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    #[allow(dead_code)] // Used in future stories for rate limiting and CORS
    pub security: SecurityConfig,
    #[allow(dead_code)] // Used in future stories for validation limits
    pub limits: LimitsConfig,
    #[allow(dead_code)] // Used in Story 8.3 for automatic path correction
    pub map_matching: MapMatchingConfig,
    /// JWT authentication configuration
    pub jwt: JwtAuthConfig,
    /// Email service configuration
    #[serde(default)]
    pub email: EmailConfig,
    /// OAuth provider configuration
    #[serde(default)]
    pub oauth: OAuthConfig,
    /// Firebase Cloud Messaging configuration
    #[serde(default)]
    pub fcm: FcmConfig,
    /// Frontend static file serving configuration
    #[serde(default)]
    pub frontend: FrontendConfig,
    /// Authentication toggles (registration, invite-only, oauth-only)
    #[serde(default)]
    pub auth_toggles: AuthTogglesConfig,
    /// Feature toggles for optional modules
    #[serde(default)]
    pub features: FeaturesConfig,
    /// Admin bootstrap configuration
    #[serde(default)]
    pub admin: AdminBootstrapConfig,
    /// Reports configuration
    #[serde(default)]
    pub reports: ReportsConfig,
    /// Cookie configuration for httpOnly authentication
    #[serde(default)]
    pub cookies: CookieConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    #[serde(default = "default_max_body_size")]
    #[allow(dead_code)] // Used in future stories for request body size limiting
    pub max_body_size: usize,

    /// Base URL for the mobile app (used in enrollment QR codes, deep links, etc.)
    #[serde(default = "default_app_base_url")]
    pub app_base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default = "default_log_format")]
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Used in future stories for rate limiting and CORS
pub struct SecurityConfig {
    #[serde(default)]
    pub cors_origins: Vec<String>,

    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,

    /// Export rate limit per hour per organization (default: 10)
    #[serde(default = "default_export_rate_limit")]
    pub export_rate_limit_per_hour: u32,

    /// Forgot password rate limit per hour per IP (default: 5)
    #[serde(default = "default_forgot_password_rate_limit")]
    pub forgot_password_rate_limit_per_hour: u32,

    /// Request verification rate limit per hour per IP (default: 3)
    #[serde(default = "default_request_verification_rate_limit")]
    pub request_verification_rate_limit_per_hour: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Used in future stories for validation limits
pub struct LimitsConfig {
    #[serde(default = "default_max_devices_per_group")]
    pub max_devices_per_group: usize,

    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,

    #[serde(default = "default_location_retention_days")]
    pub location_retention_days: u32,

    #[serde(default = "default_max_display_name_length")]
    pub max_display_name_length: usize,

    #[serde(default = "default_max_group_id_length")]
    pub max_group_id_length: usize,

    /// Maximum webhooks per device (Story 15.1)
    #[serde(default)]
    pub max_webhooks_per_device: Option<u32>,

    /// Percentage threshold for usage warnings (default: 80%)
    /// When resource usage reaches this percentage of the limit, include a warning in responses
    #[serde(default = "default_warning_threshold_percent")]
    pub warning_threshold_percent: u32,

    /// Maximum geofences per user (Epic 9: Admin Managed Users)
    #[serde(default = "default_max_geofences_per_user")]
    pub max_geofences_per_user: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Used in Story 8.3 for automatic path correction
pub struct MapMatchingConfig {
    /// Map-matching provider: osrm or valhalla
    #[serde(default = "default_map_matching_provider")]
    pub provider: String,

    /// Service URL (required if enabled)
    #[serde(default)]
    pub url: String,

    /// Request timeout in milliseconds
    #[serde(default = "default_map_matching_timeout_ms")]
    pub timeout_ms: u64,

    /// Rate limit: max requests per minute to external service
    #[serde(default = "default_map_matching_rate_limit")]
    pub rate_limit_per_minute: u32,

    /// Number of failures before circuit breaker opens
    #[serde(default = "default_circuit_breaker_failures")]
    pub circuit_breaker_failures: u32,

    /// Seconds to keep circuit breaker open before retry
    #[serde(default = "default_circuit_breaker_reset_secs")]
    pub circuit_breaker_reset_secs: u64,

    /// Whether map-matching is enabled
    #[serde(default)]
    pub enabled: bool,
}

// Default value functions
fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_request_timeout() -> u64 {
    30
}
fn default_max_body_size() -> usize {
    1_048_576
}
fn default_app_base_url() -> String {
    // Placeholder - must be configured via PM__SERVER__APP_BASE_URL for production
    "https://app.example.com".to_string()
}
fn default_max_connections() -> u32 {
    20
}
fn default_min_connections() -> u32 {
    5
}
fn default_connect_timeout() -> u64 {
    10
}
fn default_idle_timeout() -> u64 {
    600
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "json".to_string()
}
fn default_rate_limit() -> u32 {
    100
}
fn default_export_rate_limit() -> u32 {
    10 // 10 exports per hour per organization
}
fn default_forgot_password_rate_limit() -> u32 {
    5 // 5 forgot password requests per hour per IP
}
fn default_request_verification_rate_limit() -> u32 {
    3 // 3 verification requests per hour per IP
}
fn default_max_devices_per_group() -> usize {
    20
}
fn default_max_batch_size() -> usize {
    50
}
fn default_location_retention_days() -> u32 {
    30
}
fn default_max_display_name_length() -> usize {
    50
}
fn default_max_group_id_length() -> usize {
    50
}
fn default_warning_threshold_percent() -> u32 {
    80 // Warn when usage reaches 80% of limit
}
fn default_max_geofences_per_user() -> i64 {
    50 // Default maximum geofences per user
}
fn default_map_matching_provider() -> String {
    "osrm".to_string()
}
fn default_map_matching_timeout_ms() -> u64 {
    30000
}
fn default_map_matching_rate_limit() -> u32 {
    30
}
fn default_circuit_breaker_failures() -> u32 {
    5
}
fn default_circuit_breaker_reset_secs() -> u64 {
    60
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtAuthConfig {
    /// RSA private key in PEM format for signing tokens
    pub private_key: String,

    /// RSA public key in PEM format for verifying tokens
    pub public_key: String,

    /// Access token expiration in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry_secs: i64,

    /// Refresh token expiration in seconds (default: 2592000 = 30 days)
    #[serde(default = "default_refresh_token_expiry")]
    pub refresh_token_expiry_secs: i64,

    /// Leeway in seconds for clock skew tolerance (default: 30)
    /// Allows tokens to be accepted if they expired within this many seconds
    #[serde(default = "default_jwt_leeway")]
    pub leeway_secs: u64,
}

fn default_access_token_expiry() -> i64 {
    3600 // 1 hour
}

fn default_refresh_token_expiry() -> i64 {
    2592000 // 30 days
}

fn default_jwt_leeway() -> u64 {
    30 // 30 seconds for clock skew tolerance
}

/// Email service configuration for sending verification and reset emails.
#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    /// Whether email sending is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Email provider: smtp, sendgrid, ses, or console (for development)
    #[serde(default = "default_email_provider")]
    pub provider: String,

    /// SMTP server host (for smtp provider)
    #[serde(default)]
    pub smtp_host: String,

    /// SMTP server port (for smtp provider)
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,

    /// SMTP username (for smtp provider)
    #[serde(default)]
    pub smtp_username: String,

    /// SMTP password (for smtp provider)
    #[serde(default)]
    pub smtp_password: String,

    /// Whether to use TLS for SMTP (default: true)
    #[serde(default = "default_smtp_tls")]
    pub smtp_use_tls: bool,

    /// SendGrid API key (for sendgrid provider)
    #[serde(default)]
    pub sendgrid_api_key: String,

    /// AWS SES region (for ses provider)
    #[serde(default)]
    pub ses_region: String,

    /// Sender email address (From header)
    #[serde(default = "default_sender_email")]
    pub sender_email: String,

    /// Sender name (From header)
    #[serde(default = "default_sender_name")]
    pub sender_name: String,

    /// Base URL for email links (e.g., https://app.example.com)
    #[serde(default)]
    pub base_url: String,

    /// Email template style: html or plain
    #[serde(default = "default_template_style")]
    pub template_style: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_email_provider(),
            smtp_host: String::new(),
            smtp_port: default_smtp_port(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_use_tls: default_smtp_tls(),
            sendgrid_api_key: String::new(),
            ses_region: String::new(),
            sender_email: default_sender_email(),
            sender_name: default_sender_name(),
            base_url: String::new(),
            template_style: default_template_style(),
        }
    }
}

fn default_email_provider() -> String {
    "console".to_string() // Default to console logging for development
}

fn default_smtp_port() -> u16 {
    587 // TLS submission port
}

fn default_smtp_tls() -> bool {
    true
}

fn default_sender_email() -> String {
    // Placeholder - must be configured via PM__EMAIL__SENDER_EMAIL for production
    "noreply@example.com".to_string()
}

fn default_sender_name() -> String {
    "Phone Manager".to_string()
}

fn default_template_style() -> String {
    "html".to_string()
}

/// OAuth provider configuration for social login.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OAuthConfig {
    /// Google OAuth client ID (audience for token validation)
    #[serde(default)]
    pub google_client_id: String,

    /// Apple OAuth client ID / Service ID (audience for token validation)
    #[serde(default)]
    pub apple_client_id: String,

    /// Apple OAuth team ID (for server-to-server auth)
    #[serde(default)]
    pub apple_team_id: String,
}

/// Firebase Cloud Messaging configuration for push notifications.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FcmConfig {
    /// Whether FCM is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Firebase project ID
    #[serde(default)]
    pub project_id: String,

    /// Path to service account JSON file, or JSON string itself
    #[serde(default)]
    pub credentials: String,

    /// Request timeout in milliseconds
    #[serde(default = "default_fcm_timeout_ms")]
    pub timeout_ms: u64,

    /// Maximum retries for failed requests
    #[serde(default = "default_fcm_max_retries")]
    pub max_retries: u32,

    /// Whether to use high priority for data messages
    #[serde(default)]
    pub high_priority: bool,
}

fn default_fcm_timeout_ms() -> u64 {
    10000 // 10 seconds
}

fn default_fcm_max_retries() -> u32 {
    3
}

/// Authentication toggles for controlling registration and login modes.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthTogglesConfig {
    /// Whether email/password registration is enabled (default: true)
    /// When false, POST /api/v1/auth/register returns 403
    #[serde(default = "default_true")]
    pub registration_enabled: bool,

    /// Whether invite-only mode is active (default: false)
    /// When true, registration requires a valid invite token
    #[serde(default)]
    pub invite_only: bool,

    /// Whether OAuth-only mode is active (default: false)
    /// When true, disables password-based registration and login
    /// Users must use Google or Apple OAuth to sign up/in
    #[serde(default)]
    pub oauth_only: bool,
}

impl Default for AuthTogglesConfig {
    fn default() -> Self {
        Self {
            registration_enabled: true,
            invite_only: false,
            oauth_only: false,
        }
    }
}

/// Feature toggles for enabling/disabling optional modules.
/// When a feature is disabled, its endpoints return 404 "Feature not available".
#[derive(Debug, Clone, Deserialize)]
pub struct FeaturesConfig {
    /// Geofences feature (default: true)
    /// Controls: POST/GET/PATCH/DELETE /api/v1/geofences/*
    #[serde(default = "default_true")]
    pub geofences_enabled: bool,

    /// Proximity alerts feature (default: true)
    /// Controls: POST/GET/PATCH/DELETE /api/v1/proximity-alerts/*
    #[serde(default = "default_true")]
    pub proximity_alerts_enabled: bool,

    /// Webhooks feature (default: true)
    /// Controls: POST/GET/PUT/DELETE /api/v1/webhooks/*
    #[serde(default = "default_true")]
    pub webhooks_enabled: bool,

    /// Movement tracking feature - trips and movement events (default: true)
    /// Controls: /api/v1/trips/*, /api/v1/movement-events/*
    #[serde(default = "default_true")]
    pub movement_tracking_enabled: bool,

    /// B2B/Organization features (default: true)
    /// Controls: organizations, device policies, enrollment tokens, fleet management
    #[serde(default = "default_true")]
    pub b2b_enabled: bool,

    /// Geofence events feature (default: true)
    /// Controls: POST/GET /api/v1/geofence-events/*
    #[serde(default = "default_true")]
    pub geofence_events_enabled: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            geofences_enabled: true,
            proximity_alerts_enabled: true,
            webhooks_enabled: true,
            movement_tracking_enabled: true,
            b2b_enabled: true,
            geofence_events_enabled: true,
        }
    }
}

/// Admin bootstrap configuration for initial setup.
/// Allows creating the first admin user via configuration on startup.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AdminBootstrapConfig {
    /// Bootstrap admin email (if set, creates admin on first startup)
    /// Set via PM__ADMIN__BOOTSTRAP_EMAIL
    #[serde(default)]
    pub bootstrap_email: String,

    /// Bootstrap admin password (required if bootstrap_email is set)
    /// Set via PM__ADMIN__BOOTSTRAP_PASSWORD
    /// WARNING: Remove these after initial setup!
    #[serde(default)]
    pub bootstrap_password: String,
}

fn default_true() -> bool {
    true
}

/// Reports generation configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ReportsConfig {
    /// Directory to store generated reports
    #[serde(default = "default_reports_dir")]
    pub reports_dir: String,

    /// Number of reports to process per batch
    #[serde(default = "default_reports_batch_size")]
    pub batch_size: i64,

    /// Report expiration in days (default: 7)
    #[serde(default = "default_report_expiration_days")]
    pub expiration_days: u32,
}

impl Default for ReportsConfig {
    fn default() -> Self {
        Self {
            reports_dir: default_reports_dir(),
            batch_size: default_reports_batch_size(),
            expiration_days: default_report_expiration_days(),
        }
    }
}

fn default_reports_dir() -> String {
    "./reports".to_string()
}

fn default_reports_batch_size() -> i64 {
    5
}

fn default_report_expiration_days() -> u32 {
    7
}

/// Cookie configuration for httpOnly authentication.
/// Used by admin-portal for secure browser-based authentication.
#[derive(Debug, Clone, Deserialize)]
pub struct CookieConfig {
    /// Whether httpOnly cookie authentication is enabled (default: false)
    /// When true, tokens are set as httpOnly cookies instead of response body
    #[serde(default)]
    pub enabled: bool,

    /// Whether to set Secure flag on cookies (default: true)
    /// Should be true in production (HTTPS only)
    #[serde(default = "default_true")]
    pub secure: bool,

    /// SameSite policy for cookies: Strict, Lax, or None (default: Strict)
    #[serde(default = "default_same_site")]
    pub same_site: String,

    /// Cookie domain (optional, defaults to request origin)
    /// Set this if cookies need to work across subdomains
    #[serde(default)]
    pub domain: String,

    /// Path for access token cookie (default: "/")
    #[serde(default = "default_access_token_path")]
    pub access_token_path: String,

    /// Path for refresh token cookie (default: "/api/v1/auth")
    #[serde(default = "default_refresh_token_path")]
    pub refresh_token_path: String,

    /// Cookie name for access token (default: "access_token")
    #[serde(default = "default_access_token_name")]
    pub access_token_name: String,

    /// Cookie name for refresh token (default: "refresh_token")
    #[serde(default = "default_refresh_token_name")]
    pub refresh_token_name: String,
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            secure: true,
            same_site: default_same_site(),
            domain: String::new(),
            access_token_path: default_access_token_path(),
            refresh_token_path: default_refresh_token_path(),
            access_token_name: default_access_token_name(),
            refresh_token_name: default_refresh_token_name(),
        }
    }
}

fn default_same_site() -> String {
    "Strict".to_string()
}

fn default_access_token_path() -> String {
    "/".to_string()
}

fn default_refresh_token_path() -> String {
    "/api/v1/auth".to_string()
}

fn default_access_token_name() -> String {
    "access_token".to_string()
}

fn default_refresh_token_name() -> String {
    "refresh_token".to_string()
}

/// Frontend static file serving configuration for admin UI.
#[derive(Debug, Clone, Deserialize)]
pub struct FrontendConfig {
    /// Whether frontend serving is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Base directory for frontend static files
    #[serde(default = "default_frontend_base_dir")]
    pub base_dir: String,

    /// Hostname for staging environment (e.g., "admin-staging.example.com")
    #[serde(default)]
    pub staging_hostname: String,

    /// Hostname for production environment (e.g., "admin.example.com")
    #[serde(default)]
    pub production_hostname: String,

    /// Default environment when hostname doesn't match ("staging" or "production")
    #[serde(default = "default_frontend_environment")]
    pub default_environment: String,

    /// Cache max-age for immutable assets (hashed files) in seconds
    #[serde(default = "default_immutable_cache_max_age")]
    pub immutable_cache_max_age: u32,

    /// Cache max-age for mutable assets (index.html) in seconds
    #[serde(default = "default_mutable_cache_max_age")]
    pub mutable_cache_max_age: u32,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_dir: default_frontend_base_dir(),
            staging_hostname: String::new(),
            production_hostname: String::new(),
            default_environment: default_frontend_environment(),
            immutable_cache_max_age: default_immutable_cache_max_age(),
            mutable_cache_max_age: default_mutable_cache_max_age(),
        }
    }
}

fn default_frontend_base_dir() -> String {
    "/app/frontend".to_string()
}

fn default_frontend_environment() -> String {
    "production".to_string()
}

fn default_immutable_cache_max_age() -> u32 {
    31536000 // 1 year for hashed assets
}

fn default_mutable_cache_max_age() -> u32 {
    60 // 1 minute for index.html
}

/// Configuration validation error
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("Production configuration error: {0}")]
    ProductionConfig(String),
}

impl Config {
    /// Load configuration from files and environment variables.
    ///
    /// Loading order (later sources override earlier):
    /// 1. config/default.toml - base configuration with defaults
    /// 2. config/local.toml - local overrides (optional, not in git)
    /// 3. Environment variables with PM__ prefix
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("PM").separator("__"))
            .build()?;

        let cfg: Self = config.try_deserialize()?;
        cfg.validate()
            .map_err(|e| config::ConfigError::Message(e.to_string()))?;
        Ok(cfg)
    }

    /// Load configuration for testing with custom overrides.
    ///
    /// This method creates a config entirely from defaults and overrides,
    /// without relying on config files (which may not be accessible during tests).
    #[cfg(test)]
    pub fn load_for_test(overrides: &[(&str, &str)]) -> Result<Self, config::ConfigError> {
        // Embed defaults directly to avoid file system dependency in tests
        let defaults = r#"
            [server]
            host = "0.0.0.0"
            port = 8080
            request_timeout_secs = 30
            max_body_size = 1048576

            [database]
            url = ""
            max_connections = 20
            min_connections = 5
            connect_timeout_secs = 10
            idle_timeout_secs = 600

            [logging]
            level = "info"
            format = "json"

            [security]
            cors_origins = []
            rate_limit_per_minute = 100
            export_rate_limit_per_hour = 10
            forgot_password_rate_limit_per_hour = 5
            request_verification_rate_limit_per_hour = 3

            [limits]
            max_devices_per_group = 20
            max_batch_size = 50
            location_retention_days = 30
            max_display_name_length = 50
            max_group_id_length = 50
            warning_threshold_percent = 80

            [map_matching]
            provider = "osrm"
            url = ""
            timeout_ms = 30000
            rate_limit_per_minute = 30
            circuit_breaker_failures = 5
            circuit_breaker_reset_secs = 60
            enabled = false

            [jwt]
            private_key = "test-private-key"
            public_key = "test-public-key"
            access_token_expiry_secs = 3600
            refresh_token_expiry_secs = 2592000
            leeway_secs = 30

            [email]
            enabled = false
            provider = "console"
            sender_email = "test@example.com"
            sender_name = "Test"

            [oauth]
            google_client_id = ""
            apple_client_id = ""
            apple_team_id = ""

            [fcm]
            enabled = false
            project_id = ""
            credentials = ""
            timeout_ms = 10000
            max_retries = 3
            high_priority = false

            [frontend]
            enabled = false
            base_dir = "/app/frontend"
            staging_hostname = ""
            production_hostname = ""
            default_environment = "production"
            immutable_cache_max_age = 31536000
            mutable_cache_max_age = 60

            [auth_toggles]
            registration_enabled = true
            invite_only = false
            oauth_only = false

            [features]
            geofences_enabled = true
            proximity_alerts_enabled = true
            webhooks_enabled = true
            movement_tracking_enabled = true
            b2b_enabled = true
            geofence_events_enabled = true

            [admin]
            bootstrap_email = ""
            bootstrap_password = ""

            [reports]
            reports_dir = "./reports"
            batch_size = 5
            expiration_days = 7

            [cookies]
            enabled = false
            secure = true
            same_site = "Strict"
            domain = ""
            access_token_path = "/"
            refresh_token_path = "/api/v1/auth"
            access_token_name = "access_token"
            refresh_token_name = "refresh_token"
        "#;

        let mut builder = config::Config::builder()
            .add_source(config::File::from_str(defaults, config::FileFormat::Toml));

        for (key, value) in overrides {
            builder = builder.set_override(*key, *value)?;
        }

        let cfg: Self = builder.build()?.try_deserialize()?;
        // Skip validation in tests to allow partial configs
        Ok(cfg)
    }

    /// Validate configuration values.
    fn validate(&self) -> Result<(), ConfigValidationError> {
        // Database URL is required
        if self.database.url.is_empty() {
            return Err(ConfigValidationError::MissingRequired(
                "PM__DATABASE__URL environment variable must be set".to_string(),
            ));
        }

        // Validate port range
        if self.server.port == 0 {
            return Err(ConfigValidationError::InvalidValue(
                "Server port cannot be 0".to_string(),
            ));
        }

        // Validate connection pool settings
        if self.database.min_connections > self.database.max_connections {
            return Err(ConfigValidationError::InvalidValue(
                "min_connections cannot exceed max_connections".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate production-critical configuration values.
    ///
    /// This method checks for placeholder values that must be configured
    /// for production deployments. Call this at startup to catch misconfigurations.
    ///
    /// Returns warnings for non-critical issues that should be reviewed.
    pub fn validate_production(&self) -> Result<Vec<String>, ConfigValidationError> {
        let mut warnings = Vec::new();

        // Check for placeholder app_base_url
        if self.server.app_base_url == "https://app.example.com" {
            return Err(ConfigValidationError::ProductionConfig(
                "PM__SERVER__APP_BASE_URL is still set to placeholder 'https://app.example.com'. \
                 This must be configured for production to generate valid deep links and invite URLs."
                    .to_string(),
            ));
        }

        // Check for placeholder sender_email when email is enabled
        if self.email.enabled && self.email.sender_email == "noreply@example.com" {
            return Err(ConfigValidationError::ProductionConfig(
                "PM__EMAIL__SENDER_EMAIL is still set to placeholder 'noreply@example.com'. \
                 This must be configured for production email delivery."
                    .to_string(),
            ));
        }

        // Warn about email base_url when email is enabled
        if self.email.enabled && self.email.base_url.is_empty() {
            warnings.push(
                "PM__EMAIL__BASE_URL is not set. Email links will not work correctly.".to_string(),
            );
        }

        // Warn about FCM configuration when enabled
        if self.fcm.enabled {
            if self.fcm.project_id.is_empty() {
                return Err(ConfigValidationError::ProductionConfig(
                    "PM__FCM__PROJECT_ID must be set when FCM is enabled.".to_string(),
                ));
            }
            if self.fcm.credentials.is_empty() {
                return Err(ConfigValidationError::ProductionConfig(
                    "PM__FCM__CREDENTIALS must be set when FCM is enabled.".to_string(),
                ));
            }
        }

        // Warn about OAuth configuration if client IDs are empty
        if self.oauth.google_client_id.is_empty() && self.oauth.apple_client_id.is_empty() {
            warnings.push(
                "No OAuth providers configured. Social login will not be available.".to_string(),
            );
        }

        Ok(warnings)
    }

    /// Check if running with development/placeholder configuration.
    ///
    /// Returns true if any placeholder values are detected.
    pub fn is_development_config(&self) -> bool {
        self.server.app_base_url == "https://app.example.com"
            || self.email.sender_email == "noreply@example.com"
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .expect("Invalid socket address")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_with_defaults() {
        // Test loading with test overrides
        let config =
            Config::load_for_test(&[("database.url", "postgres://test:test@localhost:5432/test")])
                .expect("Failed to load config");

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_env_override() {
        let config = Config::load_for_test(&[
            ("database.url", "postgres://test:test@localhost:5432/test"),
            ("server.port", "9000"),
            ("logging.level", "debug"),
        ])
        .expect("Failed to load config");

        assert_eq!(config.server.port, 9000);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_config_validation_missing_db_url() {
        let config = Config::load_for_test(&[]).expect("Failed to load config");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("PM__DATABASE__URL"));
    }

    #[test]
    fn test_config_validation_invalid_pool_settings() {
        let config = Config::load_for_test(&[
            ("database.url", "postgres://test:test@localhost:5432/test"),
            ("database.min_connections", "100"),
            ("database.max_connections", "10"),
        ])
        .expect("Failed to load config");

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("min_connections"));
    }

    #[test]
    fn test_socket_addr() {
        let config = Config::load_for_test(&[
            ("database.url", "postgres://test:test@localhost:5432/test"),
            ("server.host", "127.0.0.1"),
            ("server.port", "3000"),
        ])
        .expect("Failed to load config");

        let addr = config.socket_addr();
        assert_eq!(addr.to_string(), "127.0.0.1:3000");
    }
}
