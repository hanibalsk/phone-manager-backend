# Phone Manager Backend - Technical Specification

**Author:** Martin Janci
**Date:** 2025-11-25
**Version:** 1.0.0
**Status:** Ready for Implementation

---

## Overview

This technical specification consolidates architecture decisions, technology choices, and implementation guidance for the Phone Manager Backend API. It serves as the definitive reference for developers implementing the 33 stories across 4 epics.

**Related Documents**:
- `PRD.md` - Product requirements and business context
- `epics.md` - Detailed user stories with acceptance criteria
- `solution-architecture.md` - Complete architectural design
- `rust-backend-spec.md` - Original technical specification (reference)

---

## Quick Reference

### Technology Stack Summary

| Layer | Technology | Version | Purpose |
|-------|------------|---------|---------|
| Language | Rust | 1.83+ (2024) | Backend implementation |
| Web Framework | Axum | 0.8 | HTTP server and routing |
| Database | PostgreSQL | 16 | Data persistence |
| Database Driver | SQLx | 0.8 | Compile-time query validation |
| Async Runtime | Tokio | 1.42 | Async execution |

### Architecture Pattern

- **Style**: Modular Monolith (single binary, workspace crates)
- **Repository**: Monorepo
- **API**: REST JSON with snake_case
- **Layers**: Routes → Services → Repositories → Database

### Performance Targets

| Metric | Target |
|--------|--------|
| API Response Time (p95) | <200ms |
| Database Query (p95) | <100ms |
| Throughput | 1,000 req/s |
| Concurrent Connections | 10,000+ |
| Uptime | 99.9% |

---

## Epic 1: Foundation & Core API Infrastructure

**Stories**: 1.1 - 1.8 (8 stories)
**Timeline**: Week 1-2
**Goal**: Production-ready infrastructure with auth, config, health monitoring

### Technical Components

#### Workspace Structure
```
crates/
├── api/           # Binary crate - HTTP layer
├── domain/        # Library - Business logic
├── persistence/   # Library - Data access
└── shared/        # Library - Utilities
```

**Cargo Workspace Configuration**:
- Resolver 2 for dependency resolution
- Workspace dependencies in root `Cargo.toml`
- Rust 1.83+ Edition 2024

#### Configuration System

**Implementation**: `crates/api/src/config.rs`

**Structure**:
```rust
pub struct Config {
    pub server: ServerConfig,      // host, port, timeouts
    pub database: DatabaseConfig,  // URL, pool settings
    pub logging: LoggingConfig,    // level, format
    pub security: SecurityConfig,  // CORS, rate limits
    pub limits: LimitsConfig,      // business limits
}
```

**Loading Priority**:
1. Environment variables (`PM__*`)
2. `config/local.toml` (developer overrides)
3. `config/default.toml` (defaults)

**Key Environment Variables**:
```bash
PM__DATABASE__URL=postgres://user:pass@host:5432/db  # Required
PM__SERVER__PORT=8080                                # Default: 8080
PM__LOGGING__LEVEL=info                              # Default: info
PM__SECURITY__RATE_LIMIT_PER_MINUTE=100             # Default: 100
PM__LIMITS__MAX_DEVICES_PER_GROUP=20                # Default: 20
```

#### Database Migrations

**Location**: `crates/persistence/src/migrations/`

**Migration Files**:
1. `001_initial.sql` - Extensions (uuid-ossp), trigger function (updated_at)
2. `002_devices.sql` - Devices table with indexes
3. `003_locations.sql` - Locations table with constraints
4. `004_api_keys.sql` - API keys table
5. `005_views_and_functions.sql` - Materialized views, cleanup function

**Applied At**: Application startup via `sqlx::migrate!()` macro

#### API Key Authentication

**Implementation**: `crates/api/src/middleware/auth.rs`

**Flow**:
1. Extract `X-API-Key` header
2. Hash with SHA-256
3. Query `api_keys` table for matching hash
4. Validate: active=true, not expired
5. Inject `ApiKeyAuth` into request state
6. Update `last_used_at` asynchronously

**Bypass**: Health endpoints (`/api/health*`)

#### Health Checks

**Endpoints**:
- `GET /api/health` - Full health with DB latency (200/503)
- `GET /api/health/live` - Process liveness (200)
- `GET /api/health/ready` - Database readiness (200/503)

**Implementation**: `crates/api/src/routes/health.rs`

---

## Epic 2: Device Management

**Stories**: 2.1 - 2.6 (6 stories)
**Timeline**: Week 2-3
**Goal**: Device registration, group membership, lifecycle management

### API Endpoints

#### POST /api/v1/devices/register

**Request**:
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "Martin's Phone",
  "group_id": "family-group-123",
  "platform": "android",
  "fcm_token": "optional-firebase-token"
}
```

**Response (200 OK)**:
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "Martin's Phone",
  "group_id": "family-group-123",
  "created_at": "2025-11-25T10:30:00Z",
  "updated_at": "2025-11-25T10:30:00Z"
}
```

**Response (409 Conflict)** - Group full:
```json
{
  "error": "conflict",
  "message": "Group has reached maximum device limit (20)"
}
```

**Implementation Notes**:
- Route: `crates/api/src/routes/devices.rs::register_device()`
- Service: `crates/domain/src/services/device.rs::DeviceService::register()`
- Repository: `crates/persistence/src/repositories/device.rs::upsert_device()`
- SQL: `INSERT ... ON CONFLICT (device_id) DO UPDATE`
- Validation: Group size check before insert

#### GET /api/v1/devices?group_id={id}

**Query Parameter**: `group_id` (required)

**Response (200 OK)**:
```json
{
  "devices": [
    {
      "device_id": "550e8400-e29b-41d4-a716-446655440000",
      "display_name": "Martin's Phone",
      "last_location": {
        "latitude": 48.1486,
        "longitude": 17.1077,
        "timestamp": "2025-11-25T10:30:00Z",
        "accuracy": 10.5
      },
      "last_seen_at": "2025-11-25T10:35:00Z"
    }
  ]
}
```

**Implementation Notes**:
- Uses `devices_with_last_location` view
- LATERAL join for last location (efficient)
- Filters: `active=true` and `group_id=?`

#### DELETE /api/v1/devices/:device_id

**Response**: 204 No Content

**Implementation Notes**:
- Soft delete: `UPDATE devices SET active=false WHERE device_id=?`
- Location records preserved
- Device excluded from group listings

### Data Model

**Database Table** (`devices`):
```sql
CREATE TABLE devices (
    id              BIGSERIAL PRIMARY KEY,
    device_id       UUID NOT NULL UNIQUE,
    display_name    VARCHAR(50) NOT NULL,
    group_id        VARCHAR(50) NOT NULL,
    platform        VARCHAR(20) NOT NULL DEFAULT 'android',
    fcm_token       TEXT,
    active          BOOLEAN NOT NULL DEFAULT true,
    last_seen_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Domain Model** (`crates/domain/src/models/device.rs`):
```rust
pub struct Device {
    pub id: i64,
    pub device_id: Uuid,
    pub display_name: String,
    pub group_id: String,
    pub platform: String,
    pub fcm_token: Option<String>,
    pub active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Validation Rules**:
- `display_name`: 2-50 characters
- `group_id`: 2-50 characters, alphanumeric + hyphens/underscores
- `platform`: Default "android"

---

## Epic 3: Location Tracking & Retrieval

**Stories**: 3.1 - 3.10 (10 stories)
**Timeline**: Week 3-5
**Goal**: Location upload, batch processing, retrieval, retention

### API Endpoints

#### POST /api/v1/locations

**Request**:
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1732527000000,
  "latitude": 48.1486,
  "longitude": 17.1077,
  "accuracy": 10.5,
  "altitude": 150.0,
  "bearing": 45.0,
  "speed": 5.2,
  "provider": "fused",
  "battery_level": 85,
  "network_type": "WiFi"
}
```

**Optional Header**: `Idempotency-Key: <uuid>`

**Response (200 OK)**:
```json
{
  "success": true,
  "processed_count": 1
}
```

**Validation**:
- Latitude: -90.0 to 90.0
- Longitude: -180.0 to 180.0
- Accuracy: >= 0.0
- Bearing: 0.0 to 360.0 (if present)
- Speed: >= 0.0 (if present)
- Battery: 0 to 100 (if present)

#### POST /api/v1/locations/batch

**Request**:
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
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

**Constraints**:
- Min 1, max 50 locations per batch
- Max payload size: 1MB
- Request timeout: 30 seconds
- Atomic transaction (all succeed or all fail)

**Response (200 OK)**:
```json
{
  "success": true,
  "processed_count": 2
}
```

### Data Model

**Database Table** (`locations`):
```sql
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

    CONSTRAINT chk_latitude CHECK (latitude >= -90 AND latitude <= 90),
    CONSTRAINT chk_longitude CHECK (longitude >= -180 AND longitude <= 180),
    CONSTRAINT chk_accuracy CHECK (accuracy >= 0),
    CONSTRAINT chk_bearing CHECK (bearing IS NULL OR (bearing >= 0 AND bearing <= 360)),
    CONSTRAINT chk_speed CHECK (speed IS NULL OR speed >= 0),
    CONSTRAINT chk_battery CHECK (battery_level IS NULL OR (battery_level >= 0 AND battery_level <= 100))
);
```

**Indexes**:
- `idx_locations_device_captured` - (device_id, captured_at DESC) for history queries
- `idx_locations_created_at` - For cleanup job
- `idx_locations_recent` - Partial index for last 24 hours

### Background Jobs

#### Location Cleanup Job

**Schedule**: Hourly (tokio interval)

**Implementation**: `crates/api/src/jobs/location_cleanup.rs`

**Query**:
```sql
DELETE FROM locations
WHERE created_at < NOW() - INTERVAL '30 days'
LIMIT 10000; -- Batched to avoid long locks
```

**Metrics**:
- `locations_deleted_total` counter
- `background_job_duration_seconds{job="cleanup"}` histogram

#### Materialized View Refresh Job

**Schedule**: Hourly

**Query**:
```sql
REFRESH MATERIALIZED VIEW CONCURRENTLY group_member_counts;
```

**Purpose**: Keep group statistics current for fast validation queries

---

## Epic 4: Production Readiness & Operational Excellence

**Stories**: 4.1 - 4.9 (9 stories)
**Timeline**: Week 5-6
**Goal**: Observability, security hardening, admin operations

### Prometheus Metrics

**Endpoint**: `GET /metrics` (no auth required)

**Metrics Exposed**:
```
# HTTP Metrics
http_requests_total{method, path, status}
http_request_duration_seconds{path}

# Database Metrics
database_connections_active
database_connections_idle
database_query_duration_seconds{query_type}

# Business Metrics
locations_uploaded_total
devices_registered_total
active_devices_count{group_id}

# Background Jobs
background_job_duration_seconds{job_name}
background_job_last_success_timestamp{job_name}
```

**Histogram Buckets**: [0.001, 0.005, 0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0]

### Rate Limiting

**Implementation**: `crates/api/src/middleware/rate_limit.rs`

**Strategy**: Sliding window per API key

**Limits**:
- Standard: 100 requests/minute
- Admin: 1000 requests/minute
- Data export: 10 requests/hour

**Library**: `governor` crate

**Response (429)**:
```json
{
  "error": "rate_limit_exceeded",
  "message": "Rate limit of 100 requests/minute exceeded",
  "retry_after": 45
}
```

**Headers**: `Retry-After: <seconds>`

### Admin Operations

#### GET /api/v1/devices/:device_id/data-export

**Purpose**: GDPR data export (FR-24)

**Auth**: Admin or device-owner API key

**Response**: Streaming JSON with device + all locations

**Rate Limit**: 10/hour

#### DELETE /api/v1/devices/:device_id/data

**Purpose**: GDPR data deletion (FR-24)

**Action**: Hard delete device + cascade all locations

**Auth**: Admin or device-owner API key

**Audit**: Log to `audit_log` table

**Response**: 204 No Content

#### DELETE /api/v1/admin/devices/inactive?older_than_days={n}

**Purpose**: Bulk cleanup (FR-26)

**Auth**: Admin API key only

**Action**: Delete devices with `active=false` older than threshold

**Response**: 200 with count

### API Key Management Tool

**Implementation**: `scripts/manage-api-key.sh` or Rust CLI binary

**Commands**:
```bash
# Generate new key
./scripts/manage-api-key.sh create --name "Prod Key"

# List keys
./scripts/manage-api-key.sh list

# Rotate key
./scripts/manage-api-key.sh rotate --prefix pm_aBcDe

# Deactivate
./scripts/manage-api-key.sh deactivate --prefix pm_aBcDe
```

**Key Format**: `pm_<45-char-base64>` (total 48 chars)

---

## Implementation Patterns

### Route → Service → Repository Pattern

**Example Flow** (Device Registration):

```rust
// 1. Route Handler
pub async fn register_device(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
    Json(payload): Json<DeviceRegistrationRequest>,
) -> Result<Json<DeviceRegistrationResponse>, ApiError> {
    payload.validate()?;
    let service = DeviceService::new(&state.pool);
    let device = service.register(payload).await?;
    Ok(Json(device.into()))
}

// 2. Service Layer
impl DeviceService {
    pub async fn register(&self, req: DeviceRegistrationRequest)
        -> Result<Device, DomainError> {
        // Validate group size
        let count = self.repo.count_active_by_group(&req.group_id).await?;
        if count >= 20 {
            return Err(DomainError::GroupFull);
        }
        // Upsert device
        self.repo.upsert_device(req).await
    }
}

// 3. Repository Layer
impl DeviceRepository {
    pub async fn upsert_device(&self, req: DeviceRegistrationRequest)
        -> Result<Device, DomainError> {
        let entity = sqlx::query_as!(
            DeviceEntity,
            "INSERT INTO devices (...) VALUES (...) ON CONFLICT DO UPDATE ..."
        )
        .fetch_one(self.pool)
        .await?;

        Ok(entity.into())
    }
}
```

### Error Handling Pattern

**Type Hierarchy**:
- `DomainError` (thiserror) - Business logic errors
- `ApiError` (thiserror) - HTTP errors
- `anyhow::Error` - Infrastructure errors

**Propagation**:
```rust
Repository → DomainError
Service → DomainError
Route → ApiError (converts DomainError via From trait)
```

**HTTP Mapping**:
```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Domain(DomainError::DeviceNotFound(_))
                => (404, "not_found", "Device not found"),
            ApiError::Domain(DomainError::GroupFull)
                => (409, "conflict", "Group full (max 20)"),
            ApiError::Unauthorized(_)
                => (401, "unauthorized", "Invalid API key"),
            ApiError::ValidationError(_)
                => (400, "validation_error", "..."),
            _ => (500, "internal_error", "Unexpected error"),
        };
        // Build JSON response
    }
}
```

### Testing Pattern

**Unit Test** (validation):
```rust
#[test]
fn test_coordinate_validation() {
    let mut payload = LocationPayload::default();
    payload.latitude = 91.0; // Invalid
    assert!(payload.validate().is_err());
}
```

**Integration Test** (full flow):
```rust
#[tokio::test]
async fn test_device_registration_and_query() {
    let server = setup_test_server().await;
    let api_key = create_test_api_key(&server).await;

    // Register
    server.post("/api/v1/devices/register")
        .add_header("X-API-Key", &api_key)
        .json(&device_payload())
        .await;

    // Query group
    let response = server.get("/api/v1/devices?group_id=test")
        .add_header("X-API-Key", &api_key)
        .await;

    assert_eq!(response.status_code(), 200);
}
```

---

## Deployment Guide

### Tier 1: Minimal (Supabase + Fly.io)

**Setup**:
1. Create Supabase project → Get PostgreSQL URL
2. `fly launch --name phone-manager-backend`
3. `fly secrets set PM__DATABASE__URL="postgres://..."`
4. `fly deploy`

**Cost**: ~$0-10/month (free tiers)
**Capacity**: <100 devices, <10K locations/day

### Tier 2: Production (Fly.io Multi-Region)

**Setup**:
1. Managed PostgreSQL (Fly.io Postgres or Supabase)
2. `fly scale count 3` (3 instances)
3. `fly regions add ams lhr fra` (Europe coverage)
4. Configure monitoring (Prometheus + Grafana)

**Cost**: ~$50-100/month
**Capacity**: 100-10K devices, 1M locations/month

### Tier 3: Enterprise (Kubernetes)

**Setup**:
1. Apply Kubernetes manifests (`k8s/*.yaml`)
2. Configure HPA (3-10 replicas)
3. Set up PostgreSQL cluster with read replicas
4. Configure monitoring stack

**Cost**: Variable (infrastructure dependent)
**Capacity**: 10K+ devices, unlimited scaling

---

## Development Commands

```bash
# Build
cargo build --workspace
cargo build --release --bin phone-manager

# Test
cargo test --workspace
cargo test --workspace -- --nocapture

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --all

# Database
sqlx database create
sqlx migrate run --source crates/persistence/src/migrations
cargo sqlx prepare --workspace

# Run
cargo run --bin phone-manager
cargo watch -x 'run --bin phone-manager'  # Auto-reload

# Docker
docker-compose up -d
docker build -t phone-manager .
```

---

## API Reference Card

| Method | Endpoint | Auth | Purpose |
|--------|----------|------|---------|
| POST | `/api/v1/devices/register` | ✓ | Register/update device |
| GET | `/api/v1/devices?group_id=` | ✓ | List group devices |
| DELETE | `/api/v1/devices/:id` | ✓ | Soft delete device |
| POST | `/api/v1/locations` | ✓ | Upload single location |
| POST | `/api/v1/locations/batch` | ✓ | Upload batch locations |
| GET | `/api/v1/devices/:id/data-export` | ✓ | GDPR data export |
| DELETE | `/api/v1/devices/:id/data` | ✓ | GDPR data deletion |
| DELETE | `/api/v1/admin/devices/inactive` | ✓ Admin | Bulk cleanup |
| GET | `/api/health` | - | Full health check |
| GET | `/api/health/live` | - | Liveness probe |
| GET | `/api/health/ready` | - | Readiness probe |
| GET | `/metrics` | - | Prometheus metrics |

---

## Security Checklist

- [ ] All endpoints require API key (except health)
- [ ] API keys hashed with SHA-256
- [ ] TLS 1.3 at load balancer
- [ ] Rate limiting per API key
- [ ] Input validation via validator crate
- [ ] SQL injection prevented (SQLx parameterized queries)
- [ ] Security headers configured
- [ ] CORS origins restricted in production
- [ ] Secrets in environment variables (never committed)
- [ ] Audit logging for admin operations
- [ ] GDPR data export/deletion endpoints

---

## Performance Checklist

- [ ] All database queries indexed
- [ ] Connection pooling configured (20 connections)
- [ ] Async I/O throughout (no blocking)
- [ ] Batch operations for bulk inserts
- [ ] Query plans validated with EXPLAIN ANALYZE
- [ ] Load testing confirms <200ms p95
- [ ] Prometheus metrics exposed
- [ ] Health checks respond quickly (<10ms)
- [ ] Background jobs non-blocking
- [ ] Graceful shutdown implemented

---

_This technical specification provides implementation-ready details for all 33 stories. Refer to solution-architecture.md for comprehensive architectural context._
