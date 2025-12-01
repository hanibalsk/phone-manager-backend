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
    "noreply@phonemanager.app".to_string()
}

fn default_sender_name() -> String {
    "Phone Manager".to_string()
}

fn default_template_style() -> String {
    "html".to_string()
}

/// Configuration validation error
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
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

            [limits]
            max_devices_per_group = 20
            max_batch_size = 50
            location_retention_days = 30
            max_display_name_length = 50
            max_group_id_length = 50

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
