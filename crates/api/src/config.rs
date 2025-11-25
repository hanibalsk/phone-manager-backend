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

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("PM").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .expect("Invalid socket address")
    }
}
