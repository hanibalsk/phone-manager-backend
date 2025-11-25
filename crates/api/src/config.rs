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
