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
- REST JSON API with camelCase field names
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

# Optional with defaults
PM__SERVER__PORT=8080
PM__LOGGING__LEVEL=info
PM__SECURITY__RATE_LIMIT_PER_MINUTE=100
PM__LIMITS__MAX_DEVICES_PER_GROUP=20
PM__LIMITS__MAX_BATCH_SIZE=50
```

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

### Authentication
- API key-based (SHA-256 hashed storage)
- Rate limiting per API key
- Key prefix for identification

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/devices/register` | Register/update device |
| GET | `/api/devices?groupId=` | List group devices with last location |
| POST | `/api/locations` | Upload single location |
| POST | `/api/locations/batch` | Upload batch locations |
| GET | `/api/health` | Health check |
| GET | `/api/health/live` | Liveness probe |
| GET | `/api/health/ready` | Readiness probe |

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
- CamelCase for JSON serialization (`#[serde(rename_all = "camelCase")]`)
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
