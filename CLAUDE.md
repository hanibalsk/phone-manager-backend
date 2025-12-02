# Phone Manager Backend - Claude Code Context

## Project Overview

Rust backend API for the Phone Manager mobile application. Handles device registration, location tracking, and group management for family/friends location sharing.

## Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | 1.83+ (Edition 2024) |
| Web Framework | Axum | 0.8 |
| Async Runtime | Tokio | 1.42 |
| Database | PostgreSQL 16 + SQLx | 0.8 |
| Validation | Validator | 0.19 |
| Serialization | Serde + JSON | 1.0 |
| Logging | Tracing | 0.1 |
| Metrics | Prometheus | 0.24 |
| Geospatial | geo | 0.28 |

## Project Structure

```
phone-manager-backend/
├── crates/
│   ├── api/           # HTTP handlers, middleware, extractors (binary)
│   ├── domain/        # Business logic, domain models, services
│   ├── persistence/   # Database layer, repositories, migrations
│   └── shared/        # Common utilities, crypto, validation
├── config/            # TOML configuration files
├── tests/             # Integration tests
└── docs/              # Specifications and design documents
```

## Architecture Pattern

**Layered Architecture** with strict separation:
- **Routes** → HTTP handlers, request/response serialization
- **Middleware** → Auth, logging, rate limiting, tracing
- **Services** → Business logic, orchestration
- **Repositories** → Data access, SQLx queries
- **Entities** → Database row mappings

## Key Design Decisions

### API Design
- REST JSON API with snake_case field names
- API key authentication via `X-API-Key` header
- Request tracing via `X-Request-ID` header
- CORS support for cross-origin requests

### Database
- SQLx for compile-time checked queries
- Migrations in `crates/persistence/src/migrations/`
- UUID for device identifiers
- Timestamps as `TIMESTAMPTZ`

### Error Handling
- `thiserror` for typed domain errors
- `anyhow` for infrastructure errors
- Structured error responses with validation details

## Configuration

Environment prefix: `PM__` (double underscore separator)

```bash
# Required
PM__DATABASE__URL=postgres://user:pass@host:5432/db
PM__JWT__PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----"
PM__JWT__PUBLIC_KEY="-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----"

# Required for production (have example.com defaults)
PM__SERVER__APP_BASE_URL=https://app.yourcompany.com  # Used for invite URLs, deep links
PM__EMAIL__SENDER_EMAIL=noreply@yourcompany.com       # Email sender address
PM__EMAIL__BASE_URL=https://app.yourcompany.com       # Links in emails

# Optional with defaults
PM__SERVER__PORT=8080
PM__LOGGING__LEVEL=info
PM__SECURITY__RATE_LIMIT_PER_MINUTE=100
PM__LIMITS__MAX_DEVICES_PER_GROUP=20
PM__LIMITS__MAX_BATCH_SIZE=50

# FCM Push Notifications (optional)
PM__FCM__ENABLED=true
PM__FCM__PROJECT_ID=your-firebase-project
PM__FCM__CREDENTIALS=/path/to/service-account.json

# OAuth Social Login (optional)
PM__OAUTH__GOOGLE_CLIENT_ID=your-google-oauth-client-id
PM__OAUTH__APPLE_CLIENT_ID=your-apple-service-id
PM__OAUTH__APPLE_TEAM_ID=your-apple-team-id

# Auth Rate Limiting (per IP, optional)
PM__SECURITY__FORGOT_PASSWORD_RATE_LIMIT_PER_HOUR=5
PM__SECURITY__REQUEST_VERIFICATION_RATE_LIMIT_PER_HOUR=3

# Admin Frontend Static File Serving (optional)
PM__FRONTEND__ENABLED=true
PM__FRONTEND__BASE_DIR=/app/frontend
PM__FRONTEND__STAGING_HOSTNAME=admin-staging.example.com
PM__FRONTEND__PRODUCTION_HOSTNAME=admin.example.com
PM__FRONTEND__DEFAULT_ENVIRONMENT=production
```

### Production Configuration Validation

The application validates configuration on startup:
- **Development mode**: If placeholder values detected (`app.example.com`, `noreply@example.com`), logs a warning
- **Production mode**: Fails to start if critical configuration is missing or invalid

Critical production requirements:
- `PM__SERVER__APP_BASE_URL` must not be `https://app.example.com`
- `PM__EMAIL__SENDER_EMAIL` must not be `noreply@example.com` (when email enabled)
- `PM__FCM__PROJECT_ID` and `PM__FCM__CREDENTIALS` required when FCM enabled

## Core Domains

### Device Management
- Registration with UUID, display name, group ID
- Group membership (max 20 devices/group)
- FCM token storage for push notifications

### Location Tracking
- Single and batch location uploads (max 50/batch)
- Coordinate validation (-90/90 lat, -180/180 lon)
- 30-day retention with automated cleanup
- Last location aggregation per device
- Location history with cursor-based pagination
- Trajectory simplification via Ramer-Douglas-Peucker algorithm
  - `tolerance` parameter (0-10000 meters)
  - Reduces points while preserving trajectory shape
  - Response includes simplification metadata

### Geofences
- Per-device circular geofences (max 50/device)
- Event types: enter, exit, dwell
- Radius: 20-50000 meters
- Optional metadata for client customization

### Proximity Alerts
- Device-to-device proximity monitoring
- Same-group constraint enforced
- Radius: 50-100000 meters
- Max 20 alerts per source device
- Haversine distance calculation (PostgreSQL function)

### Authentication
- API key-based (SHA-256 hashed storage) for device/admin APIs
- JWT-based authentication for user APIs
- OAuth social login (Google, Apple) with proper token validation
- Rate limiting per API key and per IP for auth endpoints
- Per-IP rate limiting for forgot password (5/hour) and verification (3/hour)
- Key prefix for identification

## API Endpoints

### Devices
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/devices/register` | Register/update device |
| GET | `/api/v1/devices?groupId=` | List group devices with last location |
| DELETE | `/api/v1/devices/:device_id` | Deactivate device (soft delete) |

### Locations
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/locations` | Upload single location |
| POST | `/api/v1/locations/batch` | Upload batch locations (max 50) |
| GET | `/api/v1/devices/:device_id/locations` | Get location history (cursor pagination, optional simplification) |

#### Location History Query Parameters
| Parameter | Type | Description |
|-----------|------|-------------|
| `cursor` | string | Pagination cursor from previous response |
| `limit` | int | Results per page (1-100, default 50) |
| `from` | int64 | Start timestamp (milliseconds) |
| `to` | int64 | End timestamp (milliseconds) |
| `order` | string | Sort order: `asc` or `desc` (default) |
| `tolerance` | float | Simplification tolerance in meters (0-10000). When > 0, applies RDP simplification and disables pagination |

### Geofences
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/geofences` | Create geofence |
| GET | `/api/v1/geofences?deviceId=` | List device geofences |
| GET | `/api/v1/geofences/:geofence_id` | Get geofence |
| PATCH | `/api/v1/geofences/:geofence_id` | Update geofence |
| DELETE | `/api/v1/geofences/:geofence_id` | Delete geofence |

### Proximity Alerts
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/proximity-alerts` | Create proximity alert |
| GET | `/api/v1/proximity-alerts?sourceDeviceId=` | List alerts |
| GET | `/api/v1/proximity-alerts/:alert_id` | Get alert |
| PATCH | `/api/v1/proximity-alerts/:alert_id` | Update alert |
| DELETE | `/api/v1/proximity-alerts/:alert_id` | Delete alert |

### Trips
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/trips` | Create trip with idempotency |
| PATCH | `/api/v1/trips/:trip_id` | Update trip state (complete/cancel) |
| GET | `/api/v1/devices/:device_id/trips` | List device trips (paginated) |
| GET | `/api/v1/trips/:trip_id/movement-events` | Get trip movement events |
| GET | `/api/v1/trips/:trip_id/path` | Get trip path correction data |
| POST | `/api/v1/trips/:trip_id/correct-path` | Trigger path correction |

### Movement Events
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/movement-events` | Create single movement event |
| POST | `/api/v1/movement-events/batch` | Create batch movement events (max 100) |
| GET | `/api/v1/devices/:device_id/movement-events` | Get device movement events (paginated) |
| GET | `/api/v1/trips/:trip_id/movement-events` | Get trip movement events |

### Privacy (GDPR)
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/devices/:device_id/data-export` | Export device data |
| DELETE | `/api/v1/devices/:device_id/data` | Delete all device data |

### Admin
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/admin/stats` | Get system statistics |
| DELETE | `/api/v1/admin/devices/inactive` | Delete inactive devices |
| POST | `/api/v1/admin/devices/:device_id/reactivate` | Reactivate device |

### Health
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/health/live` | Liveness probe |
| GET | `/api/health/ready` | Readiness probe |
| GET | `/metrics` | Prometheus metrics |

### Documentation
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/docs/` | Swagger UI |
| GET | `/api/docs/openapi.yaml` | OpenAPI spec |

## Development Commands

```bash
# Build
cargo build --workspace
cargo build --release --bin phone-manager

# Test
cargo test --workspace
cargo test --workspace -- --nocapture

# Lint & Format
cargo fmt --all
cargo clippy --workspace -- -D warnings

# Database
sqlx database create
sqlx migrate run --source crates/persistence/src/migrations
cargo sqlx prepare --workspace  # Generate offline query data

# Run
cargo run --bin phone-manager
```

## Performance Requirements

| Metric | Target |
|--------|--------|
| API Response Time | < 200ms (p95) |
| Uptime | 99.9% |
| Concurrent Connections | 10,000+ |
| Max Batch Size | 50 locations |

## Implementation Phases

### Phase 1: Foundation ✓
- Project structure, config, database
- API key authentication
- Device registration
- Health checks

### Phase 2: Location Tracking
- Single/batch location upload
- Group device listing
- Last location aggregation

### Phase 3: Production Readiness
- Prometheus metrics
- Rate limiting
- Security hardening
- Background jobs

## Code Conventions

### Rust Style
- Use `#[derive(Debug, Clone)]` on all public types
- Prefer `thiserror` for error types
- Use `tracing` macros for logging
- Validate inputs at API boundary with `validator`

### Naming
- Snake case for Rust identifiers
- snake_case for JSON serialization (`#[serde(rename_all = "snake_case")]`)
- Prefix environment vars with `PM__`

### Testing
- Unit tests in module `#[cfg(test)]` blocks
- Integration tests in `tests/` directory
- Use `fake` crate for test data generation
- Test database isolation per test

## Key Files Reference

- **Entry Point**: `crates/api/src/main.rs`
- **App Builder**: `crates/api/src/app.rs`
- **Configuration**: `crates/api/src/config.rs`
- **Error Types**: `crates/api/src/error.rs`
- **Migrations**: `crates/persistence/src/migrations/`
- **Spec Document**: `docs/rust-backend-spec.md`

## Dependencies Policy

- Prefer workspace dependencies in root `Cargo.toml`
- Use stable, well-maintained crates only
- Pin major versions for stability
- Document rationale for non-obvious dependencies

## Security Considerations

- Never log API keys or tokens
- Validate all user inputs
- Use parameterized queries (SQLx enforces this)
- Rate limit API endpoints
- Hash API keys with SHA-256 before storage
