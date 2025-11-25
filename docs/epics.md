# phone-manager-backend - Epic Breakdown

**Author:** Martin Janci
**Date:** 2025-11-25
**Project Level:** Level 3 (Full Product)
**Target Scale:** 32 stories across 4 epics, 6-week timeline

---

## Epic Overview

This document provides the detailed story breakdown for the Phone Manager backend. Each epic delivers incremental value and follows the technical architecture defined in `rust-backend-spec.md`.

**Epic Delivery Timeline**:
- **Week 1-2**: Epic 1 (Foundation & Core API Infrastructure) - 8 stories
- **Week 2-3**: Epic 2 (Device Management) - 6 stories
- **Week 3-5**: Epic 3 (Location Tracking & Retrieval) - 10 stories
- **Week 5-6**: Epic 4 (Production Readiness & Operational Excellence) - 8 stories

**Total**: 32 stories, 6 weeks

---

## Epic Details

### Epic 1: Foundation & Core API Infrastructure

**Epic Goal**: Establish production-ready API infrastructure with authentication, configuration, and health monitoring

**Business Value**: Enables all subsequent development; establishes security and observability baseline

**Success Criteria**:
- Rust workspace compiles and passes all lints (clippy, rustfmt)
- All endpoints enforce API key authentication
- Health checks expose database connectivity status
- Configuration system supports multiple deployment tiers
- Structured JSON logs in production mode

---

#### Story 1.1: Initialize Rust Workspace Structure

**As a** developer
**I want** a properly structured Rust workspace with all crates
**So that** I can develop features in isolated, maintainable modules

**Prerequisites**: None (first story)

**Acceptance Criteria**:
1. Workspace contains 4 crates: api (binary), domain, persistence, shared
2. Root `Cargo.toml` defines workspace dependencies (Axum 0.8, Tokio 1.42, SQLx 0.8, etc.)
3. Each crate has appropriate `Cargo.toml` with workspace dependency references
4. `rust-toolchain.toml` pins Rust version to 1.83+ with Edition 2024
5. Project compiles with `cargo build --workspace`
6. All crates pass `cargo clippy --workspace -- -D warnings`
7. Code formatted with `cargo fmt --all`

**Technical Notes**:
- Follow layered architecture: Routes → Services → Repositories → Entities
- Use workspace dependency inheritance to avoid version conflicts

---

#### Story 1.2: Configuration Management System

**As a** DevOps engineer
**I want** flexible configuration via TOML files and environment variables
**So that** I can deploy to different environments without code changes

**Prerequisites**: Story 1.1

**Acceptance Criteria**:
1. `config/default.toml` defines all configuration with sensible defaults
2. Environment variables with `PM__` prefix override TOML values (double underscore separator)
3. Configuration struct includes: server (host, port, timeouts), database (URL, pool settings), logging, security, limits
4. `.env.example` documents all available configuration options
5. Configuration loads successfully in tests with test-specific overrides
6. Missing required config (e.g., `PM__DATABASE__URL`) returns clear error message

**Technical Notes**:
- Use `config` crate for TOML + env merging
- Use `dotenvy` for `.env` file loading in development

---

#### Story 1.3: PostgreSQL Database Setup and Migrations

**As a** developer
**I want** database schema managed via SQLx migrations
**So that** schema changes are version-controlled and reproducible

**Prerequisites**: Story 1.2

**Acceptance Criteria**:
1. `crates/persistence/src/migrations/` contains numbered SQL migration files
2. Migration 001: Create uuid-ossp extension, updated_at trigger function
3. Migration 002: Create devices table with indexes
4. Migration 003: Create locations table with constraints and indexes
5. Migration 004: Create api_keys table
6. Migration 005: Create views (devices_with_last_location) and cleanup function
7. `sqlx migrate run` applies all migrations successfully
8. Database connection pool initializes with configured min/max connections

**Technical Notes**:
- Use `sqlx::migrate!()` macro for embedded migrations
- All timestamps use `TIMESTAMPTZ` for timezone awareness
- Add check constraints for coordinate ranges, battery levels

---

#### Story 1.4: API Key Authentication Middleware

**As a** security engineer
**I want** all API endpoints protected by API key authentication
**So that** only authorized clients can access the system

**Prerequisites**: Story 1.3

**Acceptance Criteria**:
1. Middleware extracts API key from `X-API-Key` header
2. SHA-256 hash computed and compared against `api_keys` table
3. Inactive or expired keys rejected with 401 Unauthorized
4. Health check endpoints (`/api/health*`) bypass authentication
5. Missing API key returns 401 with JSON error: `{"error": "unauthorized", "message": "Invalid or missing API key"}`
6. Valid API key updates `last_used_at` timestamp
7. Rate limit counter associated with authenticated API key

**Technical Notes**:
- Implement as Axum middleware using tower layers
- Use `sha2` crate for hashing
- Store authenticated key ID in request extensions for downstream use

---

#### Story 1.5: Health Check Endpoints

**As a** SRE
**I want** health check endpoints for monitoring and orchestration
**So that** I can verify system health and route traffic appropriately

**Prerequisites**: Story 1.3

**Acceptance Criteria**:
1. `GET /api/health` returns 200 with JSON: `{"status": "healthy", "version": "<version>", "database": {"connected": true, "latency_ms": <value>}}`
2. `GET /api/health/live` returns 200 if process is running (liveness probe)
3. `GET /api/health/ready` returns 200 if database is accessible (readiness probe)
4. Database connectivity failure returns 503 Service Unavailable for `/health` and `/health/ready`
5. All health endpoints bypass API key authentication
6. Health checks include latency measurement via simple `SELECT 1` query

**Technical Notes**:
- Keep health checks lightweight (<10ms execution time)
- Use for Kubernetes liveness/readiness probes

---

#### Story 1.6: Structured Logging with Tracing

**As a** developer
**I want** structured JSON logs with request tracing
**So that** I can debug issues and monitor system behavior

**Prerequisites**: Story 1.2

**Acceptance Criteria**:
1. Logs output as structured JSON in production (`PM__LOGGING__FORMAT=json`)
2. Pretty-printed logs in development for readability
3. Log level configurable via `PM__LOGGING__LEVEL` (trace, debug, info, warn, error)
4. Request tracing includes `request_id` from `X-Request-ID` header (auto-generated if missing)
5. All HTTP requests logged with: method, path, status, duration_ms, request_id
6. Database queries logged at debug level with execution time
7. Errors logged with full context and stack traces

**Technical Notes**:
- Use `tracing` and `tracing-subscriber` crates
- Configure subscriber based on environment
- Include span context for distributed tracing

---

#### Story 1.7: Error Handling Framework

**As a** API consumer
**I want** consistent error responses across all endpoints
**So that** I can handle errors predictably in my client code

**Prerequisites**: Story 1.1

**Acceptance Criteria**:
1. All errors return JSON with structure: `{"error": "<code>", "message": "<human-readable>", "details": [...]}`
2. Validation errors include `details` array with field-level errors: `[{"field": "latitude", "message": "Latitude must be between -90 and 90"}]`
3. HTTP status codes: 400 (validation), 401 (auth), 404 (not found), 409 (conflict), 429 (rate limit), 500 (server error)
4. Internal errors (500) never expose sensitive implementation details
5. Rate limit (429) includes `Retry-After` header
6. Error types use `thiserror` for domain errors, `anyhow` for infrastructure errors

**Technical Notes**:
- Implement Axum `IntoResponse` for custom error types
- Map database errors to appropriate HTTP status codes
- Log full error context while returning sanitized message to client

---

#### Story 1.8: Docker Development Environment

**As a** developer
**I want** containerized development environment
**So that** I can run the stack locally with minimal setup

**Prerequisites**: Story 1.2, Story 1.3

**Acceptance Criteria**:
1. `docker-compose.yml` defines: api service, PostgreSQL 16 service
2. API service mounts source code for live reloading
3. PostgreSQL initializes with empty database
4. `docker-compose up` starts all services successfully
5. API accessible at `http://localhost:8080`
6. Database accessible for local tools (e.g., psql, DBeaver)
7. Includes healthchecks for both services

**Technical Notes**:
- Use multi-stage Dockerfile for production builds
- Development uses `cargo watch` for auto-rebuild
- Volume mounts for Cargo cache to speed up rebuilds

---

### Epic 2: Device Management

**Epic Goal**: Enable mobile devices to register, update, and manage group membership

**Business Value**: Core prerequisite for location tracking; establishes user identity

**Success Criteria**:
- Devices can register and join groups
- Group size enforcement prevents >20 devices per group
- Device updates preserve location history
- Inactive devices excluded from active listings

---

#### Story 2.1: Device Registration API

**As a** mobile app
**I want** to register a device with the backend
**So that** the device can participate in location sharing

**Prerequisites**: Epic 1 complete

**Acceptance Criteria**:
1. `POST /api/v1/devices/register` accepts JSON: `{"deviceId": "<uuid>", "displayName": "<name>", "groupId": "<id>", "platform": "android", "fcmToken": "<optional>"}`
2. Validates display name (2-50 chars), group ID (2-50 chars, alphanumeric + hyphens/underscores)
3. Creates device record if doesn't exist; updates if exists (upsert based on deviceId)
4. Returns 200 with: `{"deviceId": "<uuid>", "displayName": "<name>", "groupId": "<id>", "createdAt": "<timestamp>", "updatedAt": "<timestamp>"}`
5. Returns 400 for validation errors with field-level details
6. Returns 409 if group has 20 devices and this is a new device joining
7. Sets `platform` to "android" if not provided
8. Updates `updated_at` and `last_seen_at` timestamps

**Technical Notes**:
- Use SQLx for compile-time checked queries
- Implement in `crates/api/src/routes/devices.rs`
- Domain model in `crates/domain/src/models/device.rs`
- Repository in `crates/persistence/src/repositories/device.rs`

---

#### Story 2.2: Device Update via Re-Registration

**As a** mobile app
**I want** to update device information by re-registering
**So that** I can change display name or FCM token without losing history

**Prerequisites**: Story 2.1

**Acceptance Criteria**:
1. Re-registration with same `deviceId` updates existing record
2. Updates: `display_name`, `fcm_token`, `updated_at`, `last_seen_at`
3. Preserves: `id`, `device_id`, `created_at`, all associated location records
4. Allows updating `group_id` if new group has capacity (implements FR-23)
5. If changing groups, validates new group size limit
6. Returns 200 with updated device information
7. Returns 409 if moving to full group (20 devices)

**Technical Notes**:
- Use `INSERT ... ON CONFLICT (device_id) DO UPDATE` for upsert
- Transaction ensures atomic group change validation

---

#### Story 2.3: Group Membership Validation

**As a** backend system
**I want** to validate group membership on device operations
**So that** business rules are enforced consistently

**Prerequisites**: Story 2.1

**Acceptance Criteria**:
1. Before device registration/update, count active devices in target group
2. Reject registration if group has 20 active devices (active=true)
3. Allow registration if group has <20 devices or device is updating within same group
4. Group count query executes in <50ms
5. Validation errors return 409 Conflict with message: "Group has reached maximum device limit (20)"
6. Inactive devices (active=false) don't count toward limit

**Technical Notes**:
- Use efficient `COUNT(*)` query with `WHERE active=true AND group_id=?`
- Consider caching group counts in Redis for high-traffic scenarios (future optimization)

---

#### Story 2.4: Device Soft Delete/Deactivation

**As a** mobile app
**I want** to deactivate a device without deleting its data
**So that** users can remove devices from active tracking while preserving history

**Prerequisites**: Story 2.1

**Acceptance Criteria**:
1. `DELETE /api/v1/devices/:deviceId` sets `active=false` instead of deleting row
2. Deactivated devices excluded from group device listings (active filter)
3. Location records for deactivated devices remain in database
4. Deactivated devices can be reactivated via re-registration
5. Returns 204 No Content on successful deactivation
6. Returns 404 if device doesn't exist

**Technical Notes**:
- Soft delete via `UPDATE devices SET active=false WHERE device_id=?`
- Location cleanup job respects retention policy regardless of device status

---

#### Story 2.5: Group Device Listing API

**As a** mobile app
**I want** to retrieve all active devices in a group
**So that** users can see who is sharing their location

**Prerequisites**: Story 2.1

**Acceptance Criteria**:
1. `GET /api/v1/devices?groupId=<id>` returns JSON: `{"devices": [{"deviceId": "<uuid>", "displayName": "<name>", "lastSeenAt": "<timestamp>"}]}`
2. Only returns active devices (active=true)
3. Sorted by `display_name` ascending
4. Returns empty array if group doesn't exist or has no active devices
5. Returns 400 if `groupId` query parameter missing
6. Query executes in <100ms for groups with 20 devices

**Technical Notes**:
- Simple query: `SELECT device_id, display_name, last_seen_at FROM devices WHERE group_id=? AND active=true ORDER BY display_name`
- Will be enhanced in Epic 3 to include last location

---

#### Story 2.6: Last Activity Timestamp Tracking

**As a** backend system
**I want** to update last_seen_at on all authenticated API calls
**So that** users know when devices were last active

**Prerequisites**: Story 1.4

**Acceptance Criteria**:
1. Every authenticated request updates `last_seen_at` to current timestamp
2. Updates occur in middleware after successful authentication
3. Update is fire-and-forget (doesn't block request processing)
4. Timestamp precision to seconds (TIMESTAMPTZ)
5. Visible in group device listings
6. No update for health check endpoints (unauthenticated)

**Technical Notes**:
- Async update in background after request completes
- Use `tokio::spawn` to avoid blocking response
- Consider batching updates for high-frequency clients (future optimization)

---

### Epic 3: Location Tracking & Retrieval

**Epic Goal**: Enable devices to upload locations and users to query group member locations

**Business Value**: Core product functionality; delivers on "peace of mind" value proposition

**Success Criteria**:
- Devices can upload single and batch locations
- Coordinate validation prevents invalid data
- Group queries include last known location for each device
- Automatic cleanup after 30 days

---

#### Story 3.1: Single Location Upload API

**As a** mobile app
**I want** to upload a single location point
**So that** my current location is visible to my group

**Prerequisites**: Epic 2 complete

**Acceptance Criteria**:
1. `POST /api/v1/locations` accepts JSON: `{"deviceId": "<uuid>", "timestamp": <ms-epoch>, "latitude": <float>, "longitude": <float>, "accuracy": <float>, "altitude": <optional>, "bearing": <optional>, "speed": <optional>, "provider": <optional>, "batteryLevel": <optional>, "networkType": <optional>}`
2. Validates: latitude (-90 to 90), longitude (-180 to 180), accuracy (>= 0), bearing (0-360 if present), speed (>= 0 if present), batteryLevel (0-100 if present)
3. Returns 400 for validation errors with field-level details
4. Returns 404 if device not registered
5. Returns 200 with: `{"success": true, "processedCount": 1}`
6. Stores location with `captured_at` from timestamp, `created_at` from server time
7. Converts timestamp from milliseconds to proper DateTime

**Technical Notes**:
- Domain model in `crates/domain/src/models/location.rs`
- Repository in `crates/persistence/src/repositories/location.rs`
- Use `validator` crate for declarative validation

---

#### Story 3.2: Batch Location Upload API

**As a** mobile app
**I want** to upload multiple locations at once
**So that** I can efficiently sync when coming back online

**Prerequisites**: Story 3.1

**Acceptance Criteria**:
1. `POST /api/v1/locations/batch` accepts JSON: `{"deviceId": "<uuid>", "locations": [<location-objects>]}`
2. Validates: 1-50 locations per batch, max 1MB payload
3. Each location validated same as single upload
4. Returns 400 if batch validation fails with details
5. Returns 404 if device not registered
6. Returns 200 with: `{"success": true, "processedCount": <count>}`
7. Request timeout: 30 seconds
8. All locations inserted in single transaction (atomic)

**Technical Notes**:
- Use SQLx batch insert: `INSERT INTO locations (...) VALUES ($1,$2,$3), ($4,$5,$6), ...`
- Transaction ensures all-or-nothing semantics
- Consider using `COPY` for larger batches (future optimization)

---

#### Story 3.3: Request Idempotency Support

**As a** mobile app
**I want** uploads to be idempotent based on a key
**So that** network retries don't create duplicate location records

**Prerequisites**: Story 3.1, Story 3.2

**Acceptance Criteria**:
1. Optional `Idempotency-Key` header accepted on location uploads
2. Key stored with location record or in separate `idempotency_keys` table
3. Duplicate key within 24 hours returns cached response (200 with same `processedCount`)
4. Duplicate detection works for both single and batch uploads
5. Keys expire/cleanup after 24 hours
6. Returns same response status and body for idempotent requests

**Technical Notes**:
- Store key hash + response in database or Redis
- Use `ON CONFLICT (idempotency_key) DO NOTHING` for simple deduplication
- Consider TTL-based cleanup job

---

#### Story 3.4: Location Validation Logic

**As a** backend system
**I want** comprehensive location validation
**So that** invalid data never enters the database

**Prerequisites**: Story 3.1

**Acceptance Criteria**:
1. Latitude validation: -90.0 to 90.0 (inclusive), returns error "Latitude must be between -90 and 90"
2. Longitude validation: -180.0 to 180.0 (inclusive), returns error "Longitude must be between -180 and 180"
3. Accuracy validation: >= 0.0, returns error "Accuracy must be non-negative"
4. Bearing validation (if present): 0.0 to 360.0 (inclusive)
5. Speed validation (if present): >= 0.0
6. Battery level validation (if present): 0 to 100 (inclusive)
7. Timestamp validation: not in future, not older than 7 days
8. Validation errors return 400 with all field errors in single response

**Technical Notes**:
- Use `validator` crate with custom validators
- Database check constraints provide defense-in-depth
- Unit tests for all validation edge cases

---

#### Story 3.5: Group Device Listing with Last Location

**As a** mobile app
**I want** group device listings to include last known location
**So that** users can see where everyone is on a map

**Prerequisites**: Story 2.5, Story 3.1

**Acceptance Criteria**:
1. `GET /api/v1/devices?groupId=<id>` enhanced to include last location
2. Response: `{"devices": [{"deviceId": "<uuid>", "displayName": "<name>", "lastLocation": {"latitude": <float>, "longitude": <float>, "timestamp": "<iso>", "accuracy": <float>}, "lastSeenAt": "<iso>"}]}`
3. `lastLocation` is null if device has no location records
4. Uses most recent location by `captured_at` timestamp
5. Query executes in <100ms for 20 devices
6. Accuracy included to show location quality

**Technical Notes**:
- Use `devices_with_last_location` view created in migrations
- LATERAL join for efficient last location lookup
- Index on (device_id, captured_at DESC) enables fast lookup

---

#### Story 3.6: Location Retention Policy Enforcement

**As a** privacy-conscious system
**I want** locations older than 30 days automatically deleted
**So that** user data doesn't accumulate indefinitely

**Prerequisites**: Story 3.1, Story 1.8 (background jobs)

**Acceptance Criteria**:
1. Background job runs hourly to delete old locations
2. Deletes locations where `created_at < NOW() - INTERVAL '30 days'`
3. Job logs count of deleted records
4. Job completes in <5 minutes for 1M+ location records
5. Uses database function `cleanup_old_locations(retention_days)` from migrations
6. Retention period configurable via `PM__LIMITS__LOCATION_RETENTION_DAYS`

**Technical Notes**:
- Use `tokio::time::interval` for scheduling
- DELETE in batches (e.g., 10K rows at a time) to avoid long locks
- Add index on `created_at` for efficient cleanup

---

#### Story 3.7: Background Job Scheduler Infrastructure

**As a** backend system
**I want** a background job scheduler
**So that** I can run periodic maintenance tasks

**Prerequisites**: Epic 1 complete

**Acceptance Criteria**:
1. Job scheduler starts on application startup
2. Supports hourly, daily job frequencies
3. Jobs run in separate tokio tasks (non-blocking)
4. Job execution logged with start/end times and results
5. Failed jobs logged with error details but don't crash application
6. Graceful shutdown waits for running jobs to complete (with timeout)

**Technical Notes**:
- Use `tokio::time::interval` for scheduling
- Implement in `crates/api/src/jobs/` module
- Initial job: location cleanup (Story 3.6)

---

#### Story 3.8: Database Query Performance Optimization

**As a** backend system
**I want** all queries to meet performance targets
**So that** API response times stay under 200ms

**Prerequisites**: Story 3.5

**Acceptance Criteria**:
1. Group device listing query: <50ms for 20 devices
2. Single location insert: <10ms
3. Batch location insert (50 locations): <100ms
4. Device registration query: <20ms
5. All queries use prepared statements (SQLx compile-time checks)
6. EXPLAIN ANALYZE shows index usage for all queries
7. Connection pool sized appropriately (20-100 connections)

**Technical Notes**:
- Review all migrations for proper indexing
- Use `EXPLAIN ANALYZE` to validate query plans
- Add covering indexes where needed
- Monitor query latency via metrics

---

#### Story 3.9: Materialized View Refresh for Group Stats

**As a** backend system
**I want** efficient group statistics via materialized views
**So that** group queries remain fast as data grows

**Prerequisites**: Story 3.6 (background jobs)

**Acceptance Criteria**:
1. `group_member_counts` materialized view refreshed hourly
2. View provides: group_id, member_count, last_activity
3. Refresh completes in <1 minute for 10K groups
4. Refresh runs as background job (non-blocking)
5. View used for group size validation queries (future optimization)

**Technical Notes**:
- Created in migration 005
- `REFRESH MATERIALIZED VIEW CONCURRENTLY group_member_counts`
- Requires UNIQUE index on group_id

---

#### Story 3.10: Location Upload Error Handling

**As a** mobile app
**I want** clear error messages for failed uploads
**So that** I can retry appropriately or alert the user

**Prerequisites**: Story 3.1, Story 1.7

**Acceptance Criteria**:
1. Device not found: 404 with `{"error": "not_found", "message": "Device not found. Please register first."}`
2. Validation errors: 400 with field-level details
3. Database timeout: 503 with `{"error": "service_unavailable", "message": "Database temporarily unavailable"}`
4. Large payload (>1MB): 413 with `{"error": "payload_too_large", "message": "Request exceeds maximum size"}`
5. Rate limit: 429 with `Retry-After` header
6. All errors logged with request_id for tracing

**Technical Notes**:
- Map SQLx errors to appropriate HTTP status
- Use custom error types from Story 1.7
- Include helpful messages without exposing internals

---

### Epic 4: Production Readiness & Operational Excellence

**Epic Goal**: Harden system for production with observability, security, and deployment automation

**Business Value**: Ensures reliability, enables troubleshooting, supports multiple deployment tiers

**Success Criteria**:
- Prometheus metrics exposed at /metrics
- Rate limiting enforces per-key limits
- Zero-downtime deployments supported
- Performance meets NFR targets

---

#### Story 4.1: Prometheus Metrics Export

**As an** SRE
**I want** Prometheus-compatible metrics exposed
**So that** I can monitor system health and performance

**Prerequisites**: Epic 3 complete

**Acceptance Criteria**:
1. `GET /metrics` returns Prometheus text format
2. Metrics include: http_requests_total (counter, labels: method, path, status), http_request_duration_seconds (histogram, p50/p90/p95/p99), database_query_duration_seconds (histogram), database_connections_active (gauge), database_connections_idle (gauge)
3. Endpoint bypasses API key authentication
4. Metrics update in real-time with request processing
5. Histogram buckets: [0.001, 0.005, 0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0]

**Technical Notes**:
- Use `metrics` and `metrics-exporter-prometheus` crates
- Instrument middleware for automatic HTTP metrics
- Custom metrics for business logic (locations uploaded, devices registered)

---

#### Story 4.2: Rate Limiting per API Key

**As a** backend system
**I want** rate limits enforced per API key
**So that** no single client can overwhelm the system

**Prerequisites**: Story 1.4

**Acceptance Criteria**:
1. Default limit: 100 requests/minute per API key (configurable via `PM__SECURITY__RATE_LIMIT_PER_MINUTE`)
2. Limit enforced using sliding window algorithm
3. Returns 429 Too Many Requests when limit exceeded
4. Response includes `Retry-After` header with seconds until reset
5. Response body: `{"error": "rate_limit_exceeded", "message": "Rate limit of 100 requests/minute exceeded", "retryAfter": <seconds>}`
6. Rate limit state stored in memory (Redis for multi-instance deployments in future)

**Technical Notes**:
- Use `governor` crate for rate limiting
- Store rate limiter keyed by API key ID
- Consider Redis-backed store for horizontal scaling

---

#### Story 4.3: API Versioning Strategy

**As a** backend system
**I want** versioned API endpoints
**So that** I can evolve the API without breaking existing clients

**Prerequisites**: Epic 1 complete

**Acceptance Criteria**:
1. All endpoints prefixed with `/api/v1/`
2. Old routes (`/api/devices`) redirect to `/api/v1/devices` with 301 Moved Permanently
3. API version included in OpenAPI/Swagger spec
4. Version documentation in README
5. Future versions (`/api/v2/`) can coexist with v1

**Technical Notes**:
- Update all route definitions to use `/api/v1/` prefix
- Axum router supports multiple version prefixes
- Document versioning strategy in architecture.md

---

#### Story 4.4: Security Headers and TLS Configuration

**As a** security engineer
**I want** security best practices enforced
**So that** the API is hardened against common attacks

**Prerequisites**: Epic 1 complete

**Acceptance Criteria**:
1. Response headers include: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `X-XSS-Protection: 1; mode=block`, `Strict-Transport-Security: max-age=31536000; includeSubDomains` (if HTTPS)
2. CORS configured via `PM__SECURITY__CORS_ORIGINS` (default: `*` for development, specific origins for production)
3. TLS 1.3 enforced in production (configure at load balancer/reverse proxy level)
4. Insecure endpoints (HTTP) redirect to HTTPS in production
5. API keys transmitted only over HTTPS in production

**Technical Notes**:
- Use `tower-http` middleware for security headers
- CORS middleware from `tower-http::cors`
- Document TLS configuration in deployment docs

---

#### Story 4.5: Kubernetes Deployment Manifests

**As a** DevOps engineer
**I want** Kubernetes manifests for production deployment
**So that** I can deploy to any Kubernetes cluster

**Prerequisites**: Story 1.8 (Docker)

**Acceptance Criteria**:
1. `k8s/deployment.yaml` defines: Deployment with 3 replicas, liveness/readiness probes, resource limits (500m CPU, 512Mi memory)
2. `k8s/service.yaml` defines ClusterIP service on port 8080
3. `k8s/configmap.yaml` defines non-sensitive config
4. `k8s/secret.yaml.example` template for sensitive values (database URL, API keys)
5. `k8s/ingress.yaml` defines Ingress with TLS termination
6. Rolling update strategy: maxUnavailable=1, maxSurge=1
7. Horizontal Pod Autoscaler (HPA) scales 3-10 replicas based on CPU >70%

**Technical Notes**:
- Liveness: `/api/health/live`, Readiness: `/api/health/ready`
- Store secrets in Kubernetes Secrets, never in Git
- Use kustomize for environment-specific overrides

---

#### Story 4.6: Load Testing and Performance Validation

**As an** SRE
**I want** load tests that validate performance targets
**So that** I can verify NFR compliance before production

**Prerequisites**: Epic 3 complete

**Acceptance Criteria**:
1. Load test script simulates: 10K concurrent connections, 1K requests/second sustained for 5 minutes
2. Test scenarios: device registration, single location upload, batch location upload (25 locations), group device listing
3. Results show: p95 latency <200ms for all endpoints, p99 latency <500ms, 0% error rate
4. Database connection pool doesn't exhaust (<100 connections used)
5. Memory usage stable (<500MB per instance)
6. Test results documented in `docs/load-test-results.md`

**Technical Notes**:
- Use `k6` or `wrk` for load testing
- Run against staging environment
- Automate via CI/CD for regression detection

---

#### Story 4.7: Admin Operations API

**As an** administrator
**I want** admin endpoints for system maintenance
**So that** I can manage devices and cleanup data

**Prerequisites**: Epic 2 complete

**Acceptance Criteria**:
1. `DELETE /api/v1/admin/devices/inactive?olderThanDays=<days>` deletes inactive devices older than threshold
2. `POST /api/v1/admin/devices/:deviceId/reactivate` reactivates soft-deleted device
3. Admin endpoints require special admin API key (separate from regular keys)
4. Returns count of affected records
5. All admin operations logged with admin key ID
6. Admin endpoints rate-limited separately (1000 req/min)

**Technical Notes**:
- Add `is_admin` flag to api_keys table
- Separate middleware for admin authentication
- Document admin operations in runbook

---

#### Story 4.8: Data Privacy Controls (Export & Deletion)

**As a** user
**I want** to export or delete my location data
**So that** I comply with my right to privacy (GDPR)

**Prerequisites**: Epic 3 complete

**Acceptance Criteria**:
1. `GET /api/v1/devices/:deviceId/data-export` returns all device data and locations as JSON
2. Export includes: device info, all location records (not just last 30 days), timestamps
3. `DELETE /api/v1/devices/:deviceId/data` deletes device and all associated locations (hard delete)
4. Deletion is irreversible; returns 204 No Content
5. Export completes in <30 seconds for 100K location records
6. Deletion cascades via foreign key constraints
7. Operations logged for audit trail

**Technical Notes**:
- Export uses streaming JSON to handle large datasets
- Deletion uses `ON DELETE CASCADE` in database schema
- Consider async job for exports if >1M locations

---

#### Story 4.9: API Key Management CLI Tool

**As an** administrator
**I want** a CLI tool to manage API keys
**So that** I can create, rotate, and revoke keys without direct database access

**Prerequisites**: Story 1.4 (API Key Authentication)

**Acceptance Criteria**:
1. CLI tool or script generates new API key with format: `pm_<45-char-base64>`
2. Computes SHA-256 hash and extracts 8-character prefix
3. Outputs: Full key (shown once), key hash (for database), key prefix, SQL INSERT statement
4. Supports key rotation: marks old key inactive, generates new key
5. Lists all existing keys with: prefix, name, active status, created date, last used date
6. Can deactivate keys by prefix or key ID
7. Tool is idempotent (re-running with same parameters safe)

**Technical Notes**:
- Can be Bash script (`scripts/generate-api-key.sh`) or Rust CLI binary
- Uses same hashing algorithm as authentication middleware (`sha2` crate)
- Script template already exists in `rust-backend-spec.md` Appendix A
- For Rust implementation: Use `clap` for CLI args, `rand` + `base64` for generation

**Example Usage**:
```bash
# Generate new key
./scripts/manage-api-key.sh create --name "Production Key"

# List all keys
./scripts/manage-api-key.sh list

# Rotate key
./scripts/manage-api-key.sh rotate --prefix pm_aBcDe

# Deactivate key
./scripts/manage-api-key.sh deactivate --prefix pm_aBcDe
```

---

## Out of Scope

_See PRD.md for complete out-of-scope features and future enhancements_

