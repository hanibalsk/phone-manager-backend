# Phone Manager - Rust Backend Specification

**Version**: 1.0.0
**Created**: 2025-11-25
**Status**: Draft
**Target Rust Edition**: 2024
**Minimum Rust Version**: 1.83.0

---

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture](#2-architecture)
3. [Project Structure](#3-project-structure)
4. [Dependencies](#4-dependencies)
5. [Configuration](#5-configuration)
6. [Data Models](#6-data-models)
7. [Database Schema](#7-database-schema)
8. [API Specification](#8-api-specification)
9. [Authentication & Authorization](#9-authentication--authorization)
10. [Error Handling](#10-error-handling)
11. [Validation](#11-validation)
12. [Background Jobs](#12-background-jobs)
13. [Observability](#13-observability)
14. [Testing Strategy](#14-testing-strategy)
15. [Deployment](#15-deployment)
16. [Security Considerations](#16-security-considerations)
17. [Performance Requirements](#17-performance-requirements)
18. [Implementation Phases](#18-implementation-phases)

**Appendices**
- [Appendix A: API Key Generation Script](#appendix-a-api-key-generation-script)
- [Appendix B: Environment Setup](#appendix-b-environment-setup)
- [Appendix C: Minimal Stack with Supabase](#appendix-c-minimal-stack-with-supabase) ⭐ **Recommended for small deployments**

---

## 1. Overview

### 1.1 Purpose

The Phone Manager Rust backend serves as the central API server for the Phone Manager mobile application. It handles:

- **Device Registration**: Managing device identity and group membership
- **Location Tracking**: Receiving, storing, and serving device location data
- **Group Management**: Organizing devices into groups for location sharing
- **Real-time Updates**: (Future) Push notifications for location updates

### 1.2 Key Requirements

| Requirement | Target |
|-------------|--------|
| API Response Time | < 200ms (p95) |
| Uptime | 99.9% |
| Max Devices per Group | 20 |
| Location Retention | 30 days |
| Concurrent Connections | 10,000+ |
| Max Batch Size | 50 locations |

### 1.3 Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | Rust 1.83+ | Performance, safety, async |
| Web Framework | Axum 0.8 | Modern, tower-based, async |
| Database | PostgreSQL 16 | Spatial queries, reliability |
| ORM | SQLx | Compile-time checked queries |
| Async Runtime | Tokio | Industry standard |
| Serialization | Serde + JSON | Compatibility with mobile |
| Validation | Validator | Declarative validation |
| Logging | Tracing | Structured, async-aware |
| Metrics | Prometheus | Industry standard |

---

## 2. Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           PHONE MANAGER BACKEND                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
│  │   Android    │    │   Android    │    │   Android    │               │
│  │    Client    │    │    Client    │    │    Client    │               │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘               │
│         │                   │                   │                        │
│         └───────────────────┼───────────────────┘                        │
│                             │                                            │
│                             ▼                                            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                      LOAD BALANCER / PROXY                        │   │
│  │                    (Nginx / Cloud LB / Traefik)                   │   │
│  └──────────────────────────────┬───────────────────────────────────┘   │
│                                 │                                        │
│                                 ▼                                        │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                         RUST API SERVER                           │   │
│  │  ┌─────────────────────────────────────────────────────────────┐ │   │
│  │  │                      Axum Router                             │ │   │
│  │  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐  │ │   │
│  │  │  │  Device   │ │ Location  │ │   Group   │ │   Health    │  │ │   │
│  │  │  │  Routes   │ │  Routes   │ │  Routes   │ │   Routes    │  │ │   │
│  │  │  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘ └──────┬──────┘  │ │   │
│  │  └────────┼─────────────┼─────────────┼──────────────┼─────────┘ │   │
│  │           │             │             │              │           │   │
│  │  ┌────────┴─────────────┴─────────────┴──────────────┴─────────┐ │   │
│  │  │                    Middleware Stack                          │ │   │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │ │   │
│  │  │  │  Auth   │ │ Logging │ │  Rate   │ │  CORS   │ │ Trace  │ │ │   │
│  │  │  │Extractor│ │         │ │ Limiter │ │         │ │   ID   │ │ │   │
│  │  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘ │ │   │
│  │  └──────────────────────────┬───────────────────────────────────┘ │   │
│  │                             │                                     │   │
│  │  ┌──────────────────────────┴───────────────────────────────────┐ │   │
│  │  │                     Service Layer                             │ │   │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐             │ │   │
│  │  │  │   Device    │ │  Location   │ │    Group    │             │ │   │
│  │  │  │   Service   │ │   Service   │ │   Service   │             │ │   │
│  │  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘             │ │   │
│  │  └─────────┼───────────────┼───────────────┼─────────────────────┘ │   │
│  │            │               │               │                       │   │
│  │  ┌─────────┴───────────────┴───────────────┴─────────────────────┐ │   │
│  │  │                    Repository Layer                            │ │   │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │ │   │
│  │  │  │   Device    │ │  Location   │ │  API Key    │              │ │   │
│  │  │  │    Repo     │ │    Repo     │ │    Repo     │              │ │   │
│  │  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘              │ │   │
│  │  └─────────┼───────────────┼───────────────┼─────────────────────┘ │   │
│  │            │               │               │                       │   │
│  └────────────┼───────────────┼───────────────┼───────────────────────┘   │
│               │               │               │                           │
│               ▼               ▼               ▼                           │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                       PostgreSQL Database                         │    │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐        │    │
│  │  │  devices  │ │ locations │ │  groups   │ │ api_keys  │        │    │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘        │    │
│  └──────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Layer Responsibilities

| Layer | Responsibility |
|-------|----------------|
| **Routes** | HTTP handlers, request parsing, response serialization |
| **Middleware** | Cross-cutting concerns: auth, logging, rate limiting |
| **Services** | Business logic, orchestration, validation |
| **Repositories** | Data access, query construction, caching |
| **Database** | Persistence, transactions, integrity |

### 2.3 Design Principles

1. **Separation of Concerns**: Clear layer boundaries
2. **Dependency Injection**: Services receive dependencies
3. **Error Propagation**: Typed errors with context
4. **Async Throughout**: Non-blocking I/O operations
5. **Type Safety**: Compile-time guarantees
6. **Zero-Copy Where Possible**: Minimize allocations

---

## 3. Project Structure

```
phone-manager-backend/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── rust-toolchain.toml           # Rust version pinning
├── .env.example                  # Environment template
├── docker-compose.yml            # Local dev environment
├── Dockerfile                    # Production container
├── sqlx-data.json               # SQLx offline mode data
│
├── crates/
│   ├── api/                      # Main API binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Entry point
│   │       ├── lib.rs            # Library exports
│   │       ├── config.rs         # Configuration loading
│   │       ├── app.rs            # Application builder
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── devices.rs    # Device endpoints
│   │       │   ├── locations.rs  # Location endpoints
│   │       │   ├── groups.rs     # Group endpoints
│   │       │   └── health.rs     # Health checks
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs       # API key authentication
│   │       │   ├── logging.rs    # Request/response logging
│   │       │   ├── rate_limit.rs # Rate limiting
│   │       │   └── trace_id.rs   # Request tracing
│   │       ├── extractors/
│   │       │   ├── mod.rs
│   │       │   ├── api_key.rs    # API key extractor
│   │       │   └── json.rs       # Validated JSON extractor
│   │       └── error.rs          # API error types
│   │
│   ├── domain/                   # Domain models & business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── device.rs     # Device domain model
│   │       │   ├── location.rs   # Location domain model
│   │       │   ├── group.rs      # Group domain model
│   │       │   └── api_key.rs    # API key domain model
│   │       ├── services/
│   │       │   ├── mod.rs
│   │       │   ├── device.rs     # Device service
│   │       │   ├── location.rs   # Location service
│   │       │   └── group.rs      # Group service
│   │       └── errors.rs         # Domain error types
│   │
│   ├── persistence/              # Database layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── db.rs             # Database connection pool
│   │       ├── repositories/
│   │       │   ├── mod.rs
│   │       │   ├── device.rs     # Device repository
│   │       │   ├── location.rs   # Location repository
│   │       │   ├── group.rs      # Group repository
│   │       │   └── api_key.rs    # API key repository
│   │       ├── entities/
│   │       │   ├── mod.rs
│   │       │   ├── device.rs     # Device entity (DB row)
│   │       │   ├── location.rs   # Location entity
│   │       │   └── api_key.rs    # API key entity
│   │       └── migrations/       # SQL migrations
│   │           ├── 001_initial.sql
│   │           ├── 002_devices.sql
│   │           ├── 003_locations.sql
│   │           └── 004_api_keys.sql
│   │
│   └── shared/                   # Shared utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── time.rs           # Time utilities
│           ├── validation.rs     # Common validators
│           └── crypto.rs         # Cryptographic utilities
│
├── tests/                        # Integration tests
│   ├── common/
│   │   └── mod.rs               # Test utilities
│   ├── api_tests.rs             # API integration tests
│   └── fixtures/                # Test data
│
└── scripts/
    ├── setup-db.sh              # Database setup
    └── generate-api-key.sh      # API key generation
```

---

## 4. Dependencies

### 4.1 Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/api",
    "crates/domain",
    "crates/persistence",
    "crates/shared",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.83"
authors = ["Phone Manager Team"]
license = "MIT"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.42", features = ["full", "tracing"] }

# Web framework
axum = { version = "0.8", features = ["macros", "tracing"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "request-id", "timeout", "compression-gzip"] }
hyper = { version = "1.5", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "migrate"] }

# Validation
validator = { version = "0.19", features = ["derive"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.11", features = ["v4", "serde"] }

# Configuration
config = "0.14"
dotenvy = "0.15"

# Logging & Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Security
argon2 = "0.5"
rand = "0.8"

# Metrics
metrics = "0.24"
metrics-exporter-prometheus = "0.16"

# Testing
tokio-test = "0.4"
fake = { version = "3.0", features = ["chrono", "uuid"] }
```

### 4.2 Individual Crate Dependencies

#### `crates/api/Cargo.toml`

```toml
[package]
name = "phone-manager-api"
version.workspace = true
edition.workspace = true

[[bin]]
name = "phone-manager"
path = "src/main.rs"

[dependencies]
domain = { path = "../domain" }
persistence = { path = "../persistence" }
shared = { path = "../shared" }

tokio.workspace = true
axum.workspace = true
axum-extra.workspace = true
tower.workspace = true
tower-http.workspace = true
serde.workspace = true
serde_json.workspace = true
validator.workspace = true
chrono.workspace = true
uuid.workspace = true
config.workspace = true
dotenvy.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
thiserror.workspace = true
metrics.workspace = true
metrics-exporter-prometheus.workspace = true

[dev-dependencies]
tokio-test.workspace = true
fake.workspace = true
```

#### `crates/domain/Cargo.toml`

```toml
[package]
name = "domain"
version.workspace = true
edition.workspace = true

[dependencies]
shared = { path = "../shared" }

serde.workspace = true
chrono.workspace = true
uuid.workspace = true
thiserror.workspace = true
validator.workspace = true
```

#### `crates/persistence/Cargo.toml`

```toml
[package]
name = "persistence"
version.workspace = true
edition.workspace = true

[dependencies]
domain = { path = "../domain" }
shared = { path = "../shared" }

sqlx.workspace = true
tokio.workspace = true
chrono.workspace = true
uuid.workspace = true
thiserror.workspace = true
tracing.workspace = true
```

#### `crates/shared/Cargo.toml`

```toml
[package]
name = "shared"
version.workspace = true
edition.workspace = true

[dependencies]
chrono.workspace = true
uuid.workspace = true
argon2.workspace = true
rand.workspace = true
thiserror.workspace = true
validator.workspace = true
```

---

## 5. Configuration

### 5.1 Configuration Structure

```rust
// crates/api/src/config.rs

use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub limits: LimitsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    /// Maximum request body size in bytes
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,

    /// Maximum connections in pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum connections in pool
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// Idle connection timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Output format (json, pretty)
    #[serde(default = "default_log_format")]
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    /// CORS allowed origins (comma-separated or "*")
    #[serde(default)]
    pub cors_origins: Vec<String>,

    /// Rate limit requests per minute per API key
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LimitsConfig {
    /// Maximum devices per group
    #[serde(default = "default_max_devices_per_group")]
    pub max_devices_per_group: usize,

    /// Maximum locations per batch upload
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,

    /// Location retention period in days
    #[serde(default = "default_location_retention_days")]
    pub location_retention_days: u32,

    /// Maximum display name length
    #[serde(default = "default_max_display_name_length")]
    pub max_display_name_length: usize,

    /// Maximum group ID length
    #[serde(default = "default_max_group_id_length")]
    pub max_group_id_length: usize,
}

// Default value functions
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8080 }
fn default_request_timeout() -> u64 { 30 }
fn default_max_body_size() -> usize { 1_048_576 } // 1MB
fn default_max_connections() -> u32 { 20 }
fn default_min_connections() -> u32 { 5 }
fn default_connect_timeout() -> u64 { 10 }
fn default_idle_timeout() -> u64 { 600 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "json".to_string() }
fn default_rate_limit() -> u32 { 100 }
fn default_max_devices_per_group() -> usize { 20 }
fn default_max_batch_size() -> usize { 50 }
fn default_location_retention_days() -> u32 { 30 }
fn default_max_display_name_length() -> usize { 50 }
fn default_max_group_id_length() -> usize { 50 }

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
```

### 5.2 Environment Variables

```bash
# .env.example

# Server Configuration
PM__SERVER__HOST=0.0.0.0
PM__SERVER__PORT=8080
PM__SERVER__REQUEST_TIMEOUT_SECS=30
PM__SERVER__MAX_BODY_SIZE=1048576

# Database Configuration
PM__DATABASE__URL=postgres://postgres:postgres@localhost:5432/phone_manager
PM__DATABASE__MAX_CONNECTIONS=20
PM__DATABASE__MIN_CONNECTIONS=5
PM__DATABASE__CONNECT_TIMEOUT_SECS=10
PM__DATABASE__IDLE_TIMEOUT_SECS=600

# Logging Configuration
PM__LOGGING__LEVEL=info
PM__LOGGING__FORMAT=json

# Security Configuration
PM__SECURITY__CORS_ORIGINS=*
PM__SECURITY__RATE_LIMIT_PER_MINUTE=100

# Limits Configuration
PM__LIMITS__MAX_DEVICES_PER_GROUP=20
PM__LIMITS__MAX_BATCH_SIZE=50
PM__LIMITS__LOCATION_RETENTION_DAYS=30
PM__LIMITS__MAX_DISPLAY_NAME_LENGTH=50
PM__LIMITS__MAX_GROUP_ID_LENGTH=50
```

### 5.3 Default Configuration File

```toml
# config/default.toml

[server]
host = "0.0.0.0"
port = 8080
request_timeout_secs = 30
max_body_size = 1048576

[database]
max_connections = 20
min_connections = 5
connect_timeout_secs = 10
idle_timeout_secs = 600

[logging]
level = "info"
format = "json"

[security]
cors_origins = ["*"]
rate_limit_per_minute = 100

[limits]
max_devices_per_group = 20
max_batch_size = 50
location_retention_days = 30
max_display_name_length = 50
max_group_id_length = 50
```

---

## 6. Data Models

### 6.1 Domain Models

```rust
// crates/domain/src/models/device.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Device registration request from mobile client
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRegistrationRequest {
    /// Client-generated UUID for the device
    pub device_id: Uuid,

    /// Human-readable device name
    #[validate(length(min = 2, max = 50, message = "Display name must be 2-50 characters"))]
    pub display_name: String,

    /// Group identifier for location sharing
    #[validate(length(min = 2, max = 50, message = "Group ID must be 2-50 characters"))]
    #[validate(custom(function = "validate_group_id"))]
    pub group_id: String,

    /// Platform identifier (always "android" for this app)
    #[serde(default = "default_platform")]
    pub platform: String,

    /// Firebase Cloud Messaging token for push notifications
    pub fcm_token: Option<String>,
}

/// Device registration response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRegistrationResponse {
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Device information returned in group queries
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub device_id: Uuid,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_location: Option<DeviceLastLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Last known location of a device
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLastLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<f32>,
}

/// Internal device domain model
#[derive(Debug, Clone)]
pub struct Device {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub fcm_token: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_platform() -> String {
    "android".to_string()
}

fn validate_group_id(group_id: &str) -> Result<(), validator::ValidationError> {
    // Allow alphanumeric characters, hyphens, and underscores
    if group_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_group_id");
        err.message = Some("Group ID can only contain alphanumeric characters, hyphens, and underscores".into());
        Err(err)
    }
}
```

```rust
// crates/domain/src/models/location.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Single location upload payload
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LocationPayload {
    /// Device UUID
    pub device_id: Uuid,

    /// Timestamp in milliseconds since epoch
    pub timestamp: i64,

    /// Latitude in degrees (-90 to 90)
    #[validate(range(min = -90.0, max = 90.0, message = "Latitude must be between -90 and 90"))]
    pub latitude: f64,

    /// Longitude in degrees (-180 to 180)
    #[validate(range(min = -180.0, max = 180.0, message = "Longitude must be between -180 and 180"))]
    pub longitude: f64,

    /// Accuracy in meters
    #[validate(range(min = 0.0, message = "Accuracy must be non-negative"))]
    pub accuracy: f32,

    /// Altitude in meters (optional)
    pub altitude: Option<f64>,

    /// Bearing in degrees 0-360 (optional)
    #[validate(range(min = 0.0, max = 360.0, message = "Bearing must be between 0 and 360"))]
    pub bearing: Option<f32>,

    /// Speed in m/s (optional)
    #[validate(range(min = 0.0, message = "Speed must be non-negative"))]
    pub speed: Option<f32>,

    /// Location provider (e.g., "gps", "fused")
    pub provider: Option<String>,

    /// Battery level 0-100 (optional)
    #[validate(range(min = 0, max = 100, message = "Battery level must be between 0 and 100"))]
    pub battery_level: Option<i32>,

    /// Network type (e.g., "WiFi", "Cellular")
    pub network_type: Option<String>,
}

/// Batch location upload payload
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LocationBatchPayload {
    /// Device UUID
    pub device_id: Uuid,

    /// List of locations (max 50)
    #[validate(length(min = 1, max = 50, message = "Batch must contain 1-50 locations"))]
    #[validate(nested)]
    pub locations: Vec<BatchLocationItem>,
}

/// Individual location in a batch (without device_id)
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BatchLocationItem {
    pub timestamp: i64,

    #[validate(range(min = -90.0, max = 90.0))]
    pub latitude: f64,

    #[validate(range(min = -180.0, max = 180.0))]
    pub longitude: f64,

    #[validate(range(min = 0.0))]
    pub accuracy: f32,

    pub altitude: Option<f64>,

    #[validate(range(min = 0.0, max = 360.0))]
    pub bearing: Option<f32>,

    #[validate(range(min = 0.0))]
    pub speed: Option<f32>,

    pub provider: Option<String>,

    #[validate(range(min = 0, max = 100))]
    pub battery_level: Option<i32>,

    pub network_type: Option<String>,
}

/// Location upload response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationUploadResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub processed_count: i32,
}

/// Internal location domain model
#[derive(Debug, Clone)]
pub struct Location {
    pub id: i64,
    pub device_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32,
    pub altitude: Option<f64>,
    pub bearing: Option<f32>,
    pub speed: Option<f32>,
    pub provider: Option<String>,
    pub battery_level: Option<i32>,
    pub network_type: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
```

```rust
// crates/domain/src/models/api_key.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API Key for authentication
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: i64,
    pub key_hash: String,
    pub key_prefix: String,  // First 8 chars for identification
    pub name: String,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// API Key creation request (admin endpoint)
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    /// Expiration in days (optional)
    pub expires_in_days: Option<u32>,
}

/// API Key creation response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyResponse {
    pub key: String,  // Full key (only shown once)
    pub key_prefix: String,
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### 6.2 API Response Models

```rust
// crates/domain/src/models/responses.rs

use serde::Serialize;
use super::device::DeviceInfo;

/// Response wrapper for list of devices
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicesResponse {
    pub devices: Vec<DeviceInfo>,
}

/// Generic error response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<ValidationErrorDetail>>,
}

/// Validation error detail
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
}

/// Health check response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: DatabaseHealth,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseHealth {
    pub connected: bool,
    pub latency_ms: Option<u64>,
}
```

---

## 7. Database Schema

### 7.1 Migration: Initial Setup

```sql
-- migrations/001_initial.sql

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable PostGIS for spatial queries (optional, for future use)
-- CREATE EXTENSION IF NOT EXISTS postgis;

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';
```

### 7.2 Migration: Devices Table

```sql
-- migrations/002_devices.sql

CREATE TABLE devices (
    id              BIGSERIAL PRIMARY KEY,
    device_id       UUID NOT NULL UNIQUE,
    display_name    VARCHAR(50) NOT NULL,
    group_id        VARCHAR(50) NOT NULL,
    platform        VARCHAR(20) NOT NULL DEFAULT 'android',
    fcm_token       TEXT,
    last_seen_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for group queries
CREATE INDEX idx_devices_group_id ON devices(group_id);

-- Index for device lookup by UUID
CREATE INDEX idx_devices_device_id ON devices(device_id);

-- Index for FCM token lookup (push notifications)
CREATE INDEX idx_devices_fcm_token ON devices(fcm_token) WHERE fcm_token IS NOT NULL;

-- Trigger to auto-update updated_at
CREATE TRIGGER update_devices_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 7.3 Migration: Locations Table

```sql
-- migrations/003_locations.sql

CREATE TABLE locations (
    id              BIGSERIAL PRIMARY KEY,
    device_id       UUID NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    latitude        DOUBLE PRECISION NOT NULL,
    longitude       DOUBLE PRECISION NOT NULL,
    accuracy        REAL NOT NULL,
    altitude        DOUBLE PRECISION,
    bearing         REAL,
    speed           REAL,
    provider        VARCHAR(50),
    battery_level   SMALLINT,
    network_type    VARCHAR(50),
    captured_at     TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT chk_latitude CHECK (latitude >= -90 AND latitude <= 90),
    CONSTRAINT chk_longitude CHECK (longitude >= -180 AND longitude <= 180),
    CONSTRAINT chk_accuracy CHECK (accuracy >= 0),
    CONSTRAINT chk_bearing CHECK (bearing IS NULL OR (bearing >= 0 AND bearing <= 360)),
    CONSTRAINT chk_speed CHECK (speed IS NULL OR speed >= 0),
    CONSTRAINT chk_battery CHECK (battery_level IS NULL OR (battery_level >= 0 AND battery_level <= 100))
);

-- Index for device location history
CREATE INDEX idx_locations_device_captured ON locations(device_id, captured_at DESC);

-- Index for time-based cleanup
CREATE INDEX idx_locations_created_at ON locations(created_at);

-- Partial index for recent locations (last 24 hours)
CREATE INDEX idx_locations_recent ON locations(device_id, captured_at DESC)
    WHERE captured_at > NOW() - INTERVAL '24 hours';
```

### 7.4 Migration: API Keys Table

```sql
-- migrations/004_api_keys.sql

CREATE TABLE api_keys (
    id              BIGSERIAL PRIMARY KEY,
    key_hash        VARCHAR(128) NOT NULL UNIQUE,
    key_prefix      VARCHAR(8) NOT NULL,
    name            VARCHAR(100) NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    last_used_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ
);

-- Index for active key lookup
CREATE INDEX idx_api_keys_active ON api_keys(key_hash) WHERE is_active = TRUE;

-- Index for key prefix identification
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
```

### 7.5 Migration: Indexes and Views

```sql
-- migrations/005_views_and_indexes.sql

-- Materialized view for group member counts (refreshed periodically)
CREATE MATERIALIZED VIEW group_member_counts AS
SELECT
    group_id,
    COUNT(*) as member_count,
    MAX(last_seen_at) as last_activity
FROM devices
GROUP BY group_id;

CREATE UNIQUE INDEX idx_group_member_counts ON group_member_counts(group_id);

-- View for devices with their last location
CREATE VIEW devices_with_last_location AS
SELECT
    d.id,
    d.device_id,
    d.display_name,
    d.group_id,
    d.platform,
    d.fcm_token,
    d.last_seen_at,
    d.created_at,
    d.updated_at,
    l.latitude as last_latitude,
    l.longitude as last_longitude,
    l.captured_at as last_location_time,
    l.accuracy as last_accuracy
FROM devices d
LEFT JOIN LATERAL (
    SELECT latitude, longitude, captured_at, accuracy
    FROM locations
    WHERE device_id = d.device_id
    ORDER BY captured_at DESC
    LIMIT 1
) l ON true;

-- Function to clean up old locations
CREATE OR REPLACE FUNCTION cleanup_old_locations(retention_days INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM locations
    WHERE created_at < NOW() - (retention_days || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
```

### 7.6 Entity Structs

```rust
// crates/persistence/src/entities/device.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct DeviceEntity {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub fcm_token: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DeviceWithLastLocation {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub fcm_token: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_latitude: Option<f64>,
    pub last_longitude: Option<f64>,
    pub last_location_time: Option<DateTime<Utc>>,
    pub last_accuracy: Option<f32>,
}
```

```rust
// crates/persistence/src/entities/location.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct LocationEntity {
    pub id: i64,
    pub device_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f32,
    pub altitude: Option<f64>,
    pub bearing: Option<f32>,
    pub speed: Option<f32>,
    pub provider: Option<String>,
    pub battery_level: Option<i16>,
    pub network_type: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
```

---

## 8. API Specification

### 8.1 Base URL and Headers

```
Base URL: https://api.phonemanager.example.com/api

Required Headers:
- X-API-Key: {api_key}
- Content-Type: application/json (for POST/PUT requests)

Optional Headers:
- X-Request-ID: {uuid} (for request tracing)
```

### 8.2 Device Endpoints

#### POST /api/devices/register

Register a new device or update existing device.

**Request:**
```json
{
  "deviceId": "550e8400-e29b-41d4-a716-446655440000",
  "displayName": "Martin's Phone",
  "groupId": "family-group-123",
  "platform": "android",
  "fcmToken": "firebase-token-optional"
}
```

**Response (200 OK):**
```json
{
  "deviceId": "550e8400-e29b-41d4-a716-446655440000",
  "displayName": "Martin's Phone",
  "groupId": "family-group-123",
  "createdAt": "2025-11-25T10:30:00Z",
  "updatedAt": "2025-11-25T10:30:00Z"
}
```

**Error Responses:**

- **400 Bad Request** - Validation error
```json
{
  "error": "validation_error",
  "message": "Request validation failed",
  "details": [
    {"field": "displayName", "message": "Display name must be 2-50 characters"}
  ]
}
```

- **401 Unauthorized** - Invalid API key
```json
{
  "error": "unauthorized",
  "message": "Invalid or missing API key"
}
```

- **409 Conflict** - Group is full
```json
{
  "error": "conflict",
  "message": "Group has reached maximum device limit (20)"
}
```

#### GET /api/devices

Get devices in a group.

**Query Parameters:**
- `groupId` (required): Group identifier

**Request:**
```
GET /api/devices?groupId=family-group-123
X-API-Key: {api_key}
```

**Response (200 OK):**
```json
{
  "devices": [
    {
      "deviceId": "550e8400-e29b-41d4-a716-446655440000",
      "displayName": "Martin's Phone",
      "lastLocation": {
        "latitude": 48.1486,
        "longitude": 17.1077,
        "timestamp": "2025-11-25T10:30:00Z",
        "accuracy": 10.5
      },
      "lastSeenAt": "2025-11-25T10:30:00Z"
    },
    {
      "deviceId": "660e8400-e29b-41d4-a716-446655440001",
      "displayName": "Jane's Phone",
      "lastLocation": null,
      "lastSeenAt": "2025-11-24T15:00:00Z"
    }
  ]
}
```

**Error Responses:**

- **400 Bad Request** - Missing groupId
```json
{
  "error": "validation_error",
  "message": "groupId query parameter is required"
}
```

### 8.3 Location Endpoints

#### POST /api/locations

Upload a single location.

**Request:**
```json
{
  "deviceId": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1732527000000,
  "latitude": 48.1486,
  "longitude": 17.1077,
  "accuracy": 10.5,
  "altitude": 150.0,
  "bearing": 45.0,
  "speed": 5.2,
  "provider": "fused",
  "batteryLevel": 85,
  "networkType": "WiFi"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "processedCount": 1
}
```

**Error Responses:**

- **400 Bad Request** - Invalid coordinates
```json
{
  "error": "validation_error",
  "message": "Request validation failed",
  "details": [
    {"field": "latitude", "message": "Latitude must be between -90 and 90"}
  ]
}
```

- **404 Not Found** - Device not registered
```json
{
  "error": "not_found",
  "message": "Device not found. Please register first."
}
```

#### POST /api/locations/batch

Upload multiple locations at once.

**Request:**
```json
{
  "deviceId": "550e8400-e29b-41d4-a716-446655440000",
  "locations": [
    {
      "timestamp": 1732527000000,
      "latitude": 48.1486,
      "longitude": 17.1077,
      "accuracy": 10.5
    },
    {
      "timestamp": 1732527300000,
      "latitude": 48.1490,
      "longitude": 17.1080,
      "accuracy": 8.0
    }
  ]
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "processedCount": 2
}
```

**Response (200 OK with partial failure):**
```json
{
  "success": true,
  "message": "2 of 3 locations processed successfully",
  "processedCount": 2
}
```

**Error Responses:**

- **400 Bad Request** - Batch too large
```json
{
  "error": "validation_error",
  "message": "Batch must contain 1-50 locations"
}
```

### 8.4 Health Endpoints

#### GET /api/health

Health check endpoint (no authentication required).

**Response (200 OK):**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "database": {
    "connected": true,
    "latencyMs": 5
  }
}
```

**Response (503 Service Unavailable):**
```json
{
  "status": "unhealthy",
  "version": "0.1.0",
  "database": {
    "connected": false,
    "latencyMs": null
  }
}
```

#### GET /api/health/ready

Readiness probe for Kubernetes.

**Response (200 OK):**
```
OK
```

#### GET /api/health/live

Liveness probe for Kubernetes.

**Response (200 OK):**
```
OK
```

### 8.5 Metrics Endpoint

#### GET /metrics

Prometheus metrics endpoint (may require separate authentication).

**Response (200 OK):**
```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="POST",endpoint="/api/locations",status="200"} 1234

# HELP http_request_duration_seconds HTTP request duration
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{endpoint="/api/locations",le="0.1"} 1000
```

---

## 9. Authentication & Authorization

### 9.1 API Key Authentication

```rust
// crates/api/src/extractors/api_key.rs

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use crate::error::ApiError;

pub struct ApiKeyAuth {
    pub api_key_id: i64,
    pub key_prefix: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiKeyAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract API key from header
        let api_key = parts
            .headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized("Missing X-API-Key header".into()))?;

        // Validate minimum key length
        if api_key.len() < 32 {
            return Err(ApiError::Unauthorized("Invalid API key format".into()));
        }

        // Get database pool from state
        let pool = parts
            .extensions
            .get::<sqlx::PgPool>()
            .ok_or(ApiError::Internal("Database pool not available".into()))?;

        // Hash the key and look it up
        let key_hash = shared::crypto::hash_api_key(api_key);

        let result = sqlx::query_as!(
            ApiKeyRecord,
            r#"
            SELECT id, key_prefix, is_active, expires_at
            FROM api_keys
            WHERE key_hash = $1
            "#,
            key_hash
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::Internal(format!("Database error: {}", e)))?;

        match result {
            Some(record) if record.is_active => {
                // Check expiration
                if let Some(expires_at) = record.expires_at {
                    if expires_at < chrono::Utc::now() {
                        return Err(ApiError::Unauthorized("API key has expired".into()));
                    }
                }

                // Update last_used_at asynchronously (fire and forget)
                let pool = pool.clone();
                let key_id = record.id;
                tokio::spawn(async move {
                    let _ = sqlx::query!(
                        "UPDATE api_keys SET last_used_at = NOW() WHERE id = $1",
                        key_id
                    )
                    .execute(&pool)
                    .await;
                });

                Ok(ApiKeyAuth {
                    api_key_id: record.id,
                    key_prefix: record.key_prefix,
                })
            }
            Some(_) => Err(ApiError::Unauthorized("API key is inactive".into())),
            None => Err(ApiError::Unauthorized("Invalid API key".into())),
        }
    }
}

#[derive(sqlx::FromRow)]
struct ApiKeyRecord {
    id: i64,
    key_prefix: String,
    is_active: bool,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

### 9.2 API Key Generation

```rust
// crates/shared/src/crypto.rs

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use rand::Rng;

const API_KEY_PREFIX: &str = "pm_";
const API_KEY_LENGTH: usize = 32;

/// Generate a new API key
pub fn generate_api_key() -> String {
    let random_bytes: [u8; API_KEY_LENGTH] = rand::thread_rng().gen();
    let key = base64::encode_config(&random_bytes, base64::URL_SAFE_NO_PAD);
    format!("{}{}", API_KEY_PREFIX, key)
}

/// Hash an API key for storage
pub fn hash_api_key(api_key: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Extract the prefix from an API key for identification
pub fn extract_key_prefix(api_key: &str) -> String {
    api_key.chars().take(8).collect()
}
```

### 9.3 Authorization Model

| Resource | Authorization Rule |
|----------|-------------------|
| Device Registration | Valid API key required |
| Get Group Devices | Valid API key + belongs to group (implicit) |
| Upload Location | Valid API key + device must be registered |
| Health Check | No authentication |
| Metrics | Optional separate auth (configurable) |

---

## 10. Error Handling

### 10.1 Error Types

```rust
// crates/api/src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<ValidationDetail>>,
}

#[derive(Debug, Serialize)]
struct ValidationDetail {
    field: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg.clone()),
            ApiError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Too many requests. Please try again later.".into(),
            ),
            ApiError::Internal(msg) => {
                // Log internal errors but don't expose details
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred".into(),
                )
            }
            ApiError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                msg.clone(),
            ),
        };

        let body = ErrorBody {
            error: error_code.into(),
            message,
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".into()),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => ApiError::Conflict("Resource already exists".into()),
                        "23503" => ApiError::NotFound("Referenced resource not found".into()),
                        _ => ApiError::Internal(format!("Database error: {}", db_err)),
                    }
                } else {
                    ApiError::Internal(format!("Database error: {}", db_err))
                }
            }
            _ => ApiError::Internal(format!("Database error: {}", err)),
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let details: Vec<ValidationDetail> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| ValidationDetail {
                    field: field.to_string(),
                    message: e.message.clone().map(|m| m.to_string()).unwrap_or_default(),
                })
            })
            .collect();

        let message = if details.len() == 1 {
            details[0].message.clone()
        } else {
            format!("{} validation errors", details.len())
        };

        // Store details in the error for later extraction
        ApiError::Validation(message)
    }
}
```

### 10.2 Domain Errors

```rust
// crates/domain/src/errors.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Group is full: maximum {0} devices allowed")]
    GroupFull(usize),

    #[error("Invalid device ID")]
    InvalidDeviceId,

    #[error("Invalid location data: {0}")]
    InvalidLocation(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

---

## 11. Validation

### 11.1 Request Validation

```rust
// crates/api/src/extractors/json.rs

use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::ApiError;

/// Validated JSON extractor
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Extract JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| ApiError::Validation(format!("Invalid JSON: {}", e)))?;

        // Validate
        value.validate()?;

        Ok(ValidatedJson(value))
    }
}
```

### 11.2 Custom Validators

```rust
// crates/shared/src/validation.rs

use validator::ValidationError;

/// Validate group ID format
pub fn validate_group_id(group_id: &str) -> Result<(), ValidationError> {
    if group_id.len() < 2 || group_id.len() > 50 {
        let mut err = ValidationError::new("length");
        err.message = Some("Group ID must be 2-50 characters".into());
        return Err(err);
    }

    if !group_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        let mut err = ValidationError::new("format");
        err.message = Some("Group ID can only contain alphanumeric characters, hyphens, and underscores".into());
        return Err(err);
    }

    Ok(())
}

/// Validate display name
pub fn validate_display_name(name: &str) -> Result<(), ValidationError> {
    let trimmed = name.trim();

    if trimmed.len() < 2 || trimmed.len() > 50 {
        let mut err = ValidationError::new("length");
        err.message = Some("Display name must be 2-50 characters".into());
        return Err(err);
    }

    // Check for control characters
    if trimmed.chars().any(|c| c.is_control()) {
        let mut err = ValidationError::new("format");
        err.message = Some("Display name cannot contain control characters".into());
        return Err(err);
    }

    Ok(())
}

/// Validate coordinates
pub fn validate_coordinates(lat: f64, lon: f64) -> Result<(), ValidationError> {
    if !(-90.0..=90.0).contains(&lat) {
        let mut err = ValidationError::new("range");
        err.message = Some("Latitude must be between -90 and 90".into());
        return Err(err);
    }

    if !(-180.0..=180.0).contains(&lon) {
        let mut err = ValidationError::new("range");
        err.message = Some("Longitude must be between -180 and 180".into());
        return Err(err);
    }

    Ok(())
}

/// Validate timestamp (not too far in past or future)
pub fn validate_timestamp(timestamp_ms: i64) -> Result<(), ValidationError> {
    let now = chrono::Utc::now().timestamp_millis();
    let max_age_ms = 7 * 24 * 60 * 60 * 1000; // 7 days
    let max_future_ms = 5 * 60 * 1000; // 5 minutes

    if timestamp_ms < now - max_age_ms {
        let mut err = ValidationError::new("range");
        err.message = Some("Timestamp is too old (max 7 days)".into());
        return Err(err);
    }

    if timestamp_ms > now + max_future_ms {
        let mut err = ValidationError::new("range");
        err.message = Some("Timestamp is in the future".into());
        return Err(err);
    }

    Ok(())
}
```

---

## 12. Background Jobs

### 12.1 Job Architecture

```rust
// crates/api/src/jobs/mod.rs

use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error};

pub mod cleanup;
pub mod refresh_views;

/// Background job scheduler
pub struct JobScheduler {
    pool: sqlx::PgPool,
    config: Arc<crate::config::Config>,
}

impl JobScheduler {
    pub fn new(pool: sqlx::PgPool, config: Arc<crate::config::Config>) -> Self {
        Self { pool, config }
    }

    /// Start all background jobs
    pub fn start(self) {
        // Location cleanup job - runs daily at 3 AM
        let pool = self.pool.clone();
        let retention_days = self.config.limits.location_retention_days;
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(24 * 60 * 60));
            loop {
                interval.tick().await;
                if let Err(e) = cleanup::cleanup_old_locations(&pool, retention_days).await {
                    error!("Location cleanup failed: {}", e);
                }
            }
        });

        // Materialized view refresh - runs every hour
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60 * 60));
            loop {
                interval.tick().await;
                if let Err(e) = refresh_views::refresh_group_counts(&pool).await {
                    error!("View refresh failed: {}", e);
                }
            }
        });

        info!("Background job scheduler started");
    }
}
```

### 12.2 Cleanup Job

```rust
// crates/api/src/jobs/cleanup.rs

use sqlx::PgPool;
use tracing::info;

/// Clean up locations older than retention period
pub async fn cleanup_old_locations(
    pool: &PgPool,
    retention_days: u32,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query_scalar!(
        "SELECT cleanup_old_locations($1)",
        retention_days as i32
    )
    .fetch_one(pool)
    .await?;

    let deleted = result.unwrap_or(0);
    info!("Cleaned up {} old location records", deleted);

    Ok(())
}
```

### 12.3 View Refresh Job

```rust
// crates/api/src/jobs/refresh_views.rs

use sqlx::PgPool;
use tracing::info;

/// Refresh materialized views
pub async fn refresh_group_counts(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY group_member_counts")
        .execute(pool)
        .await?;

    info!("Refreshed group_member_counts materialized view");
    Ok(())
}
```

---

## 13. Observability

### 13.1 Logging Configuration

```rust
// crates/api/src/logging.rs

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging(config: &crate::config::LoggingConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let subscriber = tracing_subscriber::registry().with(filter);

    match config.format.as_str() {
        "json" => {
            subscriber
                .with(fmt::layer().json())
                .init();
        }
        _ => {
            subscriber
                .with(fmt::layer().pretty())
                .init();
        }
    }
}
```

### 13.2 Request Logging Middleware

```rust
// crates/api/src/middleware/logging.rs

use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn, Span};

pub async fn request_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    // Extract request ID if present
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    if status.is_server_error() {
        warn!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            request_id = ?request_id,
            "Request completed with error"
        );
    } else {
        info!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            request_id = ?request_id,
            "Request completed"
        );
    }

    response
}
```

### 13.3 Metrics

```rust
// crates/api/src/metrics.rs

use metrics::{counter, histogram, describe_counter, describe_histogram};
use std::time::Duration;

pub fn init_metrics() {
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests"
    );
    describe_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    );
    describe_counter!(
        "locations_received_total",
        "Total number of locations received"
    );
    describe_counter!(
        "devices_registered_total",
        "Total number of device registrations"
    );
}

pub fn record_request(method: &str, endpoint: &str, status: u16, duration: Duration) {
    counter!("http_requests_total", "method" => method.to_string(), "endpoint" => endpoint.to_string(), "status" => status.to_string()).increment(1);
    histogram!("http_request_duration_seconds", "endpoint" => endpoint.to_string()).record(duration.as_secs_f64());
}

pub fn record_location_received(count: u64) {
    counter!("locations_received_total").increment(count);
}

pub fn record_device_registered() {
    counter!("devices_registered_total").increment(1);
}
```

---

## 14. Testing Strategy

### 14.1 Unit Tests

```rust
// Example unit test for validation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_group_id_valid() {
        assert!(validate_group_id("family-group-123").is_ok());
        assert!(validate_group_id("my_group").is_ok());
        assert!(validate_group_id("AB").is_ok());
    }

    #[test]
    fn test_validate_group_id_invalid() {
        assert!(validate_group_id("a").is_err()); // Too short
        assert!(validate_group_id("group with spaces").is_err());
        assert!(validate_group_id("group@special!chars").is_err());
    }

    #[test]
    fn test_validate_coordinates() {
        assert!(validate_coordinates(48.1486, 17.1077).is_ok());
        assert!(validate_coordinates(-90.0, -180.0).is_ok());
        assert!(validate_coordinates(90.0, 180.0).is_ok());
        assert!(validate_coordinates(91.0, 0.0).is_err());
        assert!(validate_coordinates(0.0, 181.0).is_err());
    }
}
```

### 14.2 Integration Tests

```rust
// tests/api_tests.rs

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;

mod common;

#[tokio::test]
async fn test_device_registration() {
    let server = common::setup_test_server().await;
    let api_key = common::create_test_api_key(&server).await;

    let response = server
        .post("/api/devices/register")
        .add_header("X-API-Key", &api_key)
        .json(&json!({
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "displayName": "Test Phone",
            "groupId": "test-group-123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["displayName"], "Test Phone");
    assert_eq!(body["groupId"], "test-group-123");
}

#[tokio::test]
async fn test_device_registration_invalid_group_id() {
    let server = common::setup_test_server().await;
    let api_key = common::create_test_api_key(&server).await;

    let response = server
        .post("/api/devices/register")
        .add_header("X-API-Key", &api_key)
        .json(&json!({
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "displayName": "Test Phone",
            "groupId": "invalid group with spaces"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_location_upload() {
    let server = common::setup_test_server().await;
    let api_key = common::create_test_api_key(&server).await;
    let device_id = common::register_test_device(&server, &api_key).await;

    let response = server
        .post("/api/locations")
        .add_header("X-API-Key", &api_key)
        .json(&json!({
            "deviceId": device_id,
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "latitude": 48.1486,
            "longitude": 17.1077,
            "accuracy": 10.5
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["processedCount"], 1);
}

#[tokio::test]
async fn test_get_group_devices() {
    let server = common::setup_test_server().await;
    let api_key = common::create_test_api_key(&server).await;

    // Register two devices in the same group
    common::register_device(&server, &api_key, "device-1", "Phone 1", "test-group").await;
    common::register_device(&server, &api_key, "device-2", "Phone 2", "test-group").await;

    let response = server
        .get("/api/devices?groupId=test-group")
        .add_header("X-API-Key", &api_key)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["devices"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_unauthorized_without_api_key() {
    let server = common::setup_test_server().await;

    let response = server
        .post("/api/devices/register")
        .json(&json!({
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "displayName": "Test Phone",
            "groupId": "test-group"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_health_check() {
    let server = common::setup_test_server().await;

    let response = server.get("/api/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "healthy");
}
```

### 14.3 Test Utilities

```rust
// tests/common/mod.rs

use sqlx::PgPool;
use phone_manager_api::{app::create_app, config::Config};

pub async fn setup_test_server() -> axum_test::TestServer {
    // Load test configuration
    let config = Config::load_test();

    // Create test database
    let pool = create_test_database(&config.database.url).await;

    // Run migrations
    sqlx::migrate!("./crates/persistence/src/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create app
    let app = create_app(config, pool);

    axum_test::TestServer::new(app).unwrap()
}

pub async fn create_test_database(base_url: &str) -> PgPool {
    let test_db_name = format!("test_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    // Connect to postgres database to create test database
    let admin_pool = PgPool::connect(&format!("{}/postgres", base_url))
        .await
        .expect("Failed to connect to postgres");

    sqlx::query(&format!("CREATE DATABASE {}", test_db_name))
        .execute(&admin_pool)
        .await
        .expect("Failed to create test database");

    // Connect to test database
    PgPool::connect(&format!("{}/{}", base_url, test_db_name))
        .await
        .expect("Failed to connect to test database")
}

pub async fn create_test_api_key(server: &axum_test::TestServer) -> String {
    // Insert API key directly into test database
    // Returns the API key string
    "pm_test_api_key_12345678901234567890".to_string()
}
```

---

## 15. Deployment

### 15.1 Dockerfile

```dockerfile
# Dockerfile

# Build stage
FROM rust:1.83-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin phone-manager

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/phone-manager /app/phone-manager

# Copy configuration
COPY config ./config

# Create non-root user
RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health/live || exit 1

# Run binary
CMD ["./phone-manager"]
```

### 15.2 Docker Compose (Development)

```yaml
# docker-compose.yml

version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - PM__DATABASE__URL=postgres://postgres:postgres@db:5432/phone_manager
      - PM__LOGGING__LEVEL=debug
      - PM__LOGGING__FORMAT=pretty
    depends_on:
      db:
        condition: service_healthy
    networks:
      - phone-manager

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=phone_manager
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - phone-manager

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    networks:
      - phone-manager

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
    networks:
      - phone-manager

volumes:
  postgres_data:
  grafana_data:

networks:
  phone-manager:
```

### 15.3 Kubernetes Manifests

```yaml
# k8s/deployment.yaml

apiVersion: apps/v1
kind: Deployment
metadata:
  name: phone-manager-api
  labels:
    app: phone-manager-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: phone-manager-api
  template:
    metadata:
      labels:
        app: phone-manager-api
    spec:
      containers:
        - name: api
          image: phone-manager-api:latest
          ports:
            - containerPort: 8080
          env:
            - name: PM__DATABASE__URL
              valueFrom:
                secretKeyRef:
                  name: phone-manager-secrets
                  key: database-url
            - name: PM__LOGGING__LEVEL
              value: "info"
          resources:
            requests:
              memory: "128Mi"
              cpu: "100m"
            limits:
              memory: "512Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /api/health/live
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /api/health/ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: phone-manager-api
spec:
  selector:
    app: phone-manager-api
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: phone-manager-api
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: phone-manager-api
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

---

## 16. Security Considerations

### 16.1 Security Checklist

| Category | Requirement | Implementation |
|----------|-------------|----------------|
| **Authentication** | API key required for all endpoints | X-API-Key header extraction |
| **Authorization** | Device must be registered | Check device exists before location upload |
| **Input Validation** | All inputs validated | Validator derive + custom validators |
| **SQL Injection** | Parameterized queries | SQLx compile-time checks |
| **Rate Limiting** | Per-API-key limits | Tower rate limit middleware |
| **HTTPS** | TLS required in production | Load balancer termination |
| **Secrets** | No secrets in code | Environment variables |
| **API Keys** | Hashed storage | SHA-256 hash before storage |
| **Logging** | No sensitive data logged | Redact API keys, coordinates |
| **Headers** | Security headers | Tower HTTP headers middleware |

### 16.2 Security Headers

```rust
// crates/api/src/middleware/security.rs

use axum::http::{header, HeaderValue};
use tower_http::set_header::SetResponseHeaderLayer;

pub fn security_headers() -> tower::ServiceBuilder<...> {
    tower::ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
}
```

### 16.3 Rate Limiting

```rust
// crates/api/src/middleware/rate_limit.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(limit_per_minute: u32) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            limit: limit_per_minute,
            window: Duration::from_secs(60),
        }
    }

    pub async fn check(&self, key: &str) -> bool {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        let entry = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old entries
        entry.retain(|&t| now.duration_since(t) < self.window);

        // Check limit
        if entry.len() >= self.limit as usize {
            return false;
        }

        // Record request
        entry.push(now);
        true
    }
}
```

---

## 17. Performance Requirements

### 17.1 Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| API Response Time (p50) | < 50ms | Prometheus histogram |
| API Response Time (p95) | < 200ms | Prometheus histogram |
| API Response Time (p99) | < 500ms | Prometheus histogram |
| Throughput | 1000 req/s | Load testing |
| Database Query Time | < 20ms | SQLx metrics |
| Memory Usage | < 512MB | Container limits |
| CPU Usage | < 500m | Container limits |
| Error Rate | < 0.1% | Prometheus counter |

### 17.2 Optimization Strategies

1. **Connection Pooling**: SQLx connection pool with appropriate limits
2. **Query Optimization**: Indexed queries, materialized views
3. **Response Compression**: Gzip compression for responses
4. **Async I/O**: Non-blocking operations throughout
5. **Batch Processing**: Batch location inserts
6. **Caching**: In-memory caching for frequently accessed data

### 17.3 Database Optimization

```sql
-- Batch insert for locations (used in batch upload)
INSERT INTO locations (device_id, latitude, longitude, accuracy, altitude, bearing, speed, provider, battery_level, network_type, captured_at)
SELECT * FROM UNNEST(
    $1::uuid[],
    $2::double precision[],
    $3::double precision[],
    $4::real[],
    $5::double precision[],
    $6::real[],
    $7::real[],
    $8::varchar[],
    $9::smallint[],
    $10::varchar[],
    $11::timestamptz[]
);
```

---

## 18. Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Goal**: Core infrastructure and device registration

- [ ] Project setup with workspace structure
- [ ] Configuration management
- [ ] Database connection and migrations
- [ ] API key authentication
- [ ] Device registration endpoint
- [ ] Health check endpoints
- [ ] Basic logging and error handling
- [ ] Unit tests for core components
- [ ] Docker development environment

**Deliverables**:
- Working device registration API
- Database schema for devices
- Authentication middleware
- Docker Compose for local development

### Phase 2: Location Tracking (Week 3-4)

**Goal**: Location upload and retrieval

- [ ] Single location upload endpoint
- [ ] Batch location upload endpoint
- [ ] Location storage with validation
- [ ] Get group devices endpoint
- [ ] Last location aggregation
- [ ] Integration tests
- [ ] Performance optimization for batch inserts

**Deliverables**:
- Complete location API
- Group device listing with last locations
- Full API test coverage

### Phase 3: Production Readiness (Week 5-6)

**Goal**: Observability, security hardening, deployment

- [ ] Prometheus metrics integration
- [ ] Structured logging with trace IDs
- [ ] Rate limiting
- [ ] Security headers
- [ ] Input validation hardening
- [ ] Background job scheduler
- [ ] Location cleanup job
- [ ] Kubernetes manifests
- [ ] Load testing and optimization
- [ ] Documentation

**Deliverables**:
- Production-ready deployment artifacts
- Monitoring dashboards
- API documentation
- Performance benchmarks

### Phase 4: Future Enhancements (Backlog)

- Push notification integration (FCM)
- WebSocket support for real-time updates
- Admin dashboard API
- Analytics and reporting
- Geofencing support
- Multi-tenant API keys

---

## Appendix A: API Key Generation Script

```bash
#!/bin/bash
# scripts/generate-api-key.sh

# Generate a random API key
API_KEY="pm_$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 43)"

# Hash for storage
KEY_HASH=$(echo -n "$API_KEY" | sha256sum | cut -d' ' -f1)

# Extract prefix
KEY_PREFIX="${API_KEY:0:8}"

echo "API Key (save this - shown only once):"
echo "$API_KEY"
echo ""
echo "Key Hash (for database):"
echo "$KEY_HASH"
echo ""
echo "Key Prefix:"
echo "$KEY_PREFIX"
echo ""
echo "SQL to insert:"
echo "INSERT INTO api_keys (key_hash, key_prefix, name, is_active) VALUES ('$KEY_HASH', '$KEY_PREFIX', 'Generated Key', true);"
```

---

## Appendix B: Environment Setup

```bash
# Development environment setup

# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# 2. Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# 3. Start PostgreSQL
docker-compose up -d db

# 4. Create database
sqlx database create

# 5. Run migrations
sqlx migrate run --source crates/persistence/src/migrations

# 6. Generate SQLx offline data
cargo sqlx prepare --workspace

# 7. Run tests
cargo test

# 8. Run server
cargo run --bin phone-manager
```

---

## Appendix C: Minimal Stack with Supabase

This appendix describes a simplified deployment for small-scale use (family/friends, < 100 devices) using Supabase as the managed PostgreSQL database.

### C.1 Minimal Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    MINIMAL PHONE MANAGER BACKEND                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
│  │   Android    │    │   Android    │    │   Android    │               │
│  │    Client    │    │    Client    │    │    Client    │               │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘               │
│         │                   │                   │                        │
│         └───────────────────┼───────────────────┘                        │
│                             │                                            │
│                             │ HTTPS                                      │
│                             ▼                                            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    RUST API SERVER (Single Instance)              │   │
│  │                                                                    │   │
│  │    ┌─────────────────────────────────────────────────────────┐    │   │
│  │    │  Hosting Options:                                        │    │   │
│  │    │  • fly.io (free tier: 3 VMs, 256MB each)                │    │   │
│  │    │  • Railway ($5/mo)                                       │    │   │
│  │    │  • DigitalOcean Droplet ($6/mo)                         │    │   │
│  │    │  • Hetzner VPS (€4/mo)                                  │    │   │
│  │    │  • Home server / Raspberry Pi                           │    │   │
│  │    └─────────────────────────────────────────────────────────┘    │   │
│  │                                                                    │   │
│  └──────────────────────────────┬───────────────────────────────────┘   │
│                                 │                                        │
│                                 │ PostgreSQL Connection (SSL)            │
│                                 ▼                                        │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                         SUPABASE                                  │   │
│  │                                                                    │   │
│  │    ┌─────────────────────────────────────────────────────────┐    │   │
│  │    │  Free Tier Includes:                                     │    │   │
│  │    │  • 500 MB database                                       │    │   │
│  │    │  • Unlimited API requests                                │    │   │
│  │    │  • 1 GB file storage                                     │    │   │
│  │    │  • 50,000 monthly active users                          │    │   │
│  │    │  • Automatic backups (7 days)                           │    │   │
│  │    │  • Dashboard for data inspection                        │    │   │
│  │    └─────────────────────────────────────────────────────────┘    │   │
│  │                                                                    │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### C.2 What You DON'T Need

| Component | Full Stack | Minimal Stack |
|-----------|------------|---------------|
| Load Balancer | ✅ Required | ❌ Not needed |
| Multiple Replicas | ✅ 3+ instances | ❌ Single instance |
| Kubernetes | ✅ K8s manifests | ❌ Simple Docker/binary |
| Prometheus/Grafana | ✅ Full monitoring | ❌ Basic logging |
| Redis/Cache | ✅ Optional | ❌ Not needed |
| Self-managed PostgreSQL | ✅ Required | ❌ Supabase handles it |
| CI/CD Pipeline | ✅ Full pipeline | ❌ Manual deploy OK |
| Auto-scaling | ✅ HPA | ❌ Fixed resources |

### C.3 Supabase Setup

#### Step 1: Create Supabase Project

1. Go to [supabase.com](https://supabase.com)
2. Create a new project
3. Note your project credentials:
   - **Project URL**: `https://xxxxx.supabase.co`
   - **Database URL**: `postgres://postgres.[project-ref]:[password]@aws-0-[region].pooler.supabase.com:6543/postgres`
   - **Anon Key**: (not needed for Rust backend)
   - **Service Role Key**: (not needed for Rust backend)

#### Step 2: Configure Connection String

```bash
# .env for minimal deployment
PM__DATABASE__URL=postgres://postgres.[project-ref]:[password]@aws-0-[region].pooler.supabase.com:6543/postgres?sslmode=require

# Supabase-specific settings
PM__DATABASE__MAX_CONNECTIONS=5        # Free tier has limited connections
PM__DATABASE__MIN_CONNECTIONS=1
PM__DATABASE__CONNECT_TIMEOUT_SECS=30  # Higher for remote DB
```

#### Step 3: Run Migrations via Supabase Dashboard

Option A - SQL Editor in Supabase Dashboard:
```sql
-- Copy-paste each migration file into the SQL editor
-- Run them in order: 001_initial.sql, 002_devices.sql, etc.
```

Option B - SQLx CLI with remote connection:
```bash
# Set DATABASE_URL environment variable
export DATABASE_URL="postgres://postgres.[ref]:[pass]@aws-0-[region].pooler.supabase.com:6543/postgres?sslmode=require"

# Run migrations
sqlx migrate run --source crates/persistence/src/migrations
```

### C.4 Simplified Configuration

```toml
# config/minimal.toml

[server]
host = "0.0.0.0"
port = 8080
request_timeout_secs = 30
max_body_size = 1048576

[database]
# Connection pool - keep small for Supabase free tier
max_connections = 5
min_connections = 1
connect_timeout_secs = 30
idle_timeout_secs = 300

[logging]
level = "info"
format = "pretty"  # Use pretty for single instance (easier to read)

[security]
cors_origins = ["*"]
rate_limit_per_minute = 60  # Lower for minimal setup

[limits]
max_devices_per_group = 20
max_batch_size = 50
location_retention_days = 30
max_display_name_length = 50
max_group_id_length = 50
```

### C.5 Simplified Deployment Options

#### Option A: fly.io (Recommended - Free Tier)

```toml
# fly.toml
app = "phone-manager-api"
primary_region = "fra"  # Frankfurt, choose closest to you

[build]
  dockerfile = "Dockerfile"

[env]
  PM__LOGGING__LEVEL = "info"
  PM__LOGGING__FORMAT = "pretty"
  PM__DATABASE__MAX_CONNECTIONS = "5"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = true   # Saves resources when idle
  auto_start_machines = true
  min_machines_running = 0    # Can scale to zero

[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 256
```

**Deploy commands:**
```bash
# Install fly CLI
curl -L https://fly.io/install.sh | sh

# Login
fly auth login

# Create app
fly launch

# Set secrets
fly secrets set PM__DATABASE__URL="postgres://..."

# Deploy
fly deploy

# View logs
fly logs
```

#### Option B: Railway (Simple, $5/mo)

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and deploy
railway login
railway init
railway up

# Set environment variables in Railway dashboard
# PM__DATABASE__URL = your Supabase connection string
```

#### Option C: Single VPS with Docker

```bash
# On your VPS (DigitalOcean, Hetzner, etc.)

# 1. Install Docker
curl -fsSL https://get.docker.com | sh

# 2. Pull and run
docker run -d \
  --name phone-manager \
  --restart unless-stopped \
  -p 8080:8080 \
  -e PM__DATABASE__URL="postgres://..." \
  -e PM__LOGGING__LEVEL="info" \
  ghcr.io/your-username/phone-manager:latest

# 3. Setup HTTPS with Caddy (automatic SSL)
docker run -d \
  --name caddy \
  --restart unless-stopped \
  -p 80:80 \
  -p 443:443 \
  -v caddy_data:/data \
  caddy caddy reverse-proxy --from api.yourdomain.com --to localhost:8080
```

#### Option D: Raspberry Pi / Home Server

```bash
# Cross-compile for ARM64
cross build --release --target aarch64-unknown-linux-gnu

# Copy binary to Pi
scp target/aarch64-unknown-linux-gnu/release/phone-manager pi@raspberrypi:~/

# On Raspberry Pi - create systemd service
sudo tee /etc/systemd/system/phone-manager.service << 'EOF'
[Unit]
Description=Phone Manager API
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi
Environment="PM__DATABASE__URL=postgres://..."
Environment="PM__LOGGING__LEVEL=info"
ExecStart=/home/pi/phone-manager
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl enable phone-manager
sudo systemctl start phone-manager

# Use Cloudflare Tunnel for HTTPS (free)
# Or use ngrok for testing
```

### C.6 Simplified Dockerfile

```dockerfile
# Dockerfile.minimal - Optimized for small deployments

# Build stage
FROM rust:1.83-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release --bin phone-manager

# Runtime stage - using distroless for minimal size
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/phone-manager /
COPY --from=builder /app/config /config
EXPOSE 8080
CMD ["/phone-manager"]
```

**Build for minimal size:**
```bash
# Build with size optimizations
RUSTFLAGS="-C link-arg=-s" cargo build --release

# Expected binary size: ~5-10 MB
# Docker image size: ~20-30 MB
```

### C.7 Supabase-Specific Considerations

#### Connection Pooling

Supabase uses PgBouncer for connection pooling. Use the **pooler URL** (port 6543) instead of direct connection (port 5432):

```
# ✅ Use pooler URL (recommended)
postgres://postgres.[ref]:[pass]@aws-0-[region].pooler.supabase.com:6543/postgres

# ❌ Direct connection (limited connections)
postgres://postgres.[ref]:[pass]@db.[ref].supabase.co:5432/postgres
```

#### Free Tier Limits

| Resource | Free Tier Limit | Impact |
|----------|-----------------|--------|
| Database size | 500 MB | ~5M location records |
| API requests | Unlimited | No limit |
| Bandwidth | 2 GB/month | Sufficient for < 100 devices |
| Pausing | After 7 days inactive | Wake up on first request |
| Connections | ~20 direct | Use pooler URL |

**Storage estimation:**
```
Per location record: ~100 bytes
Daily locations per device (5 min interval): 288
Monthly per device: ~8,640 records = ~864 KB
100 devices × 30 days = ~86 MB/month

With 30-day retention: ~86 MB active data
500 MB limit = ~5.8 months of data for 100 devices
```

#### Avoiding Database Pausing

Free tier databases pause after 7 days of inactivity. To prevent this:

```rust
// Add a keep-alive job (runs every 6 days)
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(6 * 24 * 60 * 60));
    loop {
        interval.tick().await;
        let _ = sqlx::query("SELECT 1").execute(&pool).await;
        tracing::info!("Database keep-alive ping");
    }
});
```

Or use an external cron service (UptimeRobot, cron-job.org) to hit your `/api/health` endpoint daily.

### C.8 Cost Comparison

| Setup | Monthly Cost | Best For |
|-------|--------------|----------|
| **Supabase Free + fly.io Free** | $0 | Development, < 20 devices |
| **Supabase Free + fly.io Hobby** | $5 | Small family group |
| **Supabase Free + Railway** | $5 | Easy deployment |
| **Supabase Free + DigitalOcean** | $6 | More control |
| **Supabase Pro + VPS** | $25 + $6 | 100+ devices |
| **Full Stack (self-managed)** | $50+ | Production scale |

### C.9 Quick Start for Minimal Setup

```bash
# 1. Create Supabase project at supabase.com
#    Note your database URL

# 2. Clone and build
git clone https://github.com/your-username/phone-manager-backend
cd phone-manager-backend
cargo build --release

# 3. Run migrations
export DATABASE_URL="postgres://postgres.[ref]:[pass]@aws-0-[region].pooler.supabase.com:6543/postgres?sslmode=require"
sqlx migrate run --source crates/persistence/src/migrations

# 4. Generate API key
./scripts/generate-api-key.sh
# Copy the SQL output and run it in Supabase SQL Editor

# 5. Deploy to fly.io
fly launch
fly secrets set PM__DATABASE__URL="$DATABASE_URL"
fly deploy

# 6. Test
curl https://your-app.fly.dev/api/health

# 7. Configure Android app with:
#    - API_BASE_URL: https://your-app.fly.dev
#    - API_KEY: pm_xxxxx (from step 4)
```

### C.10 Minimal vs Full Stack Decision Matrix

| Criteria | Choose Minimal | Choose Full Stack |
|----------|----------------|-------------------|
| Users | < 100 devices | 100+ devices |
| Budget | $0-10/month | $50+/month |
| SLA | Best effort | 99.9% uptime |
| Ops experience | Limited | Experienced |
| Location updates | Every 5+ minutes | Real-time |
| Data retention | 30 days | Longer |
| Compliance | Personal use | Business/GDPR |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-11-25 | Claude | Initial specification |
| 1.1.0 | 2025-11-25 | Claude | Added minimal stack with Supabase (Appendix C) |

---

**Last Updated**: 2025-11-25
**Status**: Draft - Ready for Review
