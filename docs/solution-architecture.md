# Phone Manager Backend - Solution Architecture

**Author:** Martin Janci
**Date:** 2025-11-25
**Project Level:** Level 3 (Full Product)
**Architecture Style:** Modular Monolith
**Repository Strategy:** Monorepo

---

## Architecture Pattern

### Style: Modular Monolith

**Decision**: Single Rust binary with workspace-based modular organization

**Rationale**:
- **Simplicity**: Single deployment unit reduces operational complexity for small teams (1-3 developers)
- **Performance**: In-process communication eliminates network latency between modules
- **Type Safety**: Shared types across crates provide compile-time guarantees
- **Deployment Flexibility**: Scales from single container (Tier 1) to multi-instance Kubernetes (Tier 3) without architectural changes
- **Future Evolution**: Clear crate boundaries enable extraction to microservices if needed (unlikely given scale)

### Repository Strategy: Monorepo

**Decision**: Single Git repository with Cargo workspace

**Rationale**:
- **Atomic Changes**: Cross-crate refactoring in single commit
- **Dependency Management**: Workspace-level dependency versioning prevents conflicts
- **CI/CD Simplicity**: Single pipeline builds all crates
- **Code Sharing**: Easy sharing of domain models, utilities across crates

### Communication Pattern

- **API Protocol**: REST over HTTP/1.1 (HTTP/2 via load balancer optional)
- **Serialization**: JSON with **snake_case** field naming
- **Content Type**: `application/json`
- **Internal**: Direct function calls between layers (no RPC)

### Key Architectural Constraints

1. **Layered Architecture**: Strict unidirectional dependencies (Routes → Services → Repositories)
2. **Database-Centric**: PostgreSQL as single source of truth
3. **Async Throughout**: All I/O operations non-blocking via Tokio
4. **Stateless API**: No session state; scales horizontally
5. **Compile-Time Safety**: SQLx for query validation, strong typing everywhere

## Component Boundaries

### Crate Organization (Workspace Structure)

#### 1. `crates/api` (Binary Crate)
- **Epics**: Epic 1 (Foundation), Epic 4 (Production - observability)
- **Responsibilities**: HTTP layer, routing, middleware, request/response handling
- **Domain**: Routes, extractors, middleware (auth, logging, rate limiting, tracing, CORS)
- **Dependencies**: domain, persistence, shared
- **Entry Point**: `src/main.rs` (binary), `src/app.rs` (application builder)

#### 2. `crates/domain` (Library Crate)
- **Epics**: Epic 2 (Device Management), Epic 3 (Location Tracking)
- **Responsibilities**: Business logic, validation, domain models, service orchestration
- **Domain**: Device service, Location service, Group service, validation logic
- **Dependencies**: shared (no persistence dependency - pure business logic)
- **Key Principle**: Framework-agnostic, testable without database

#### 3. `crates/persistence` (Library Crate)
- **Epics**: Epic 1 (Database), Epic 3 (Location storage)
- **Responsibilities**: Database access, migrations, entity mapping, repository pattern
- **Domain**: Device repository, Location repository, API key repository, entities, migrations
- **Dependencies**: domain, shared
- **Key Principle**: SQLx compile-time query validation, repository abstraction

#### 4. `crates/shared` (Library Crate)
- **Epics**: Cross-cutting (all epics)
- **Responsibilities**: Common utilities, cryptography, time handling, validation helpers
- **Domain**: API key hashing (SHA-256), time utilities, custom validators
- **Dependencies**: None (bottom of dependency tree)
- **Key Principle**: Zero business logic, pure utilities

### Shared Infrastructure Components

#### Authentication Module (`api/middleware/auth.rs`)
- **Epic**: Epic 1, used by all subsequent epics
- **Responsibilities**: API key extraction, hashing, validation, rate limiting
- **Pattern**: Axum middleware with extractor pattern

#### Background Jobs (`api/jobs/`)
- **Epic**: Epic 3 (retention cleanup), Epic 4 (materialized view refresh)
- **Responsibilities**: Scheduled task execution via tokio intervals
- **Pattern**: Async task spawning with graceful shutdown

### Integration Points

| Integration | Direction | Protocol | Authentication |
|-------------|-----------|----------|----------------|
| Mobile Client → API | Inbound | REST/JSON over HTTPS | X-API-Key header |
| API → PostgreSQL | Outbound | PostgreSQL wire protocol | Connection string credentials |
| Prometheus → API | Inbound | HTTP GET /metrics | None (may add basic auth) |
| Future: API → FCM | Outbound | HTTPS/JSON | FCM server key (out of scope) |

### Dependency Graph

```
api (binary)
├─ domain
│  └─ shared
├─ persistence
│  ├─ domain
│  │  └─ shared
│  └─ shared
└─ shared

(All dependencies flow downward, no circular dependencies)
```

## Architecture Decisions

### Service Architecture
- **Pattern**: Modular Monolith (single Rust binary)
- **Repository**: Monorepo with Cargo workspace
- **Rationale**: Simplicity for small team, eliminates network overhead, type-safe boundaries, easy local development

### API Design
- **Paradigm**: REST over HTTP
- **Serialization**: JSON with **snake_case** field naming
- **Versioning**: URL-based (`/api/v1/`) for backward compatibility (FR-25)
- **Documentation**: OpenAPI/Swagger spec generation (future), README with examples (MVP)
- **Rationale**: snake_case aligns with Rust conventions, REST provides simplicity and wide tooling support

### Communication Patterns
- **Client ↔ API**: Synchronous HTTP request/response
- **API ↔ Database**: Async via SQLx connection pool
- **Internal Layers**: Direct function calls (in-process, zero-copy where possible)
- **Background Jobs**: Async tokio tasks with interval-based scheduling
- **Rationale**: Synchronous HTTP sufficient for location sharing use case, async I/O for performance

### Database and Data Layer
- **Primary Database**: PostgreSQL 16
- **Access Pattern**: Direct SQL via SQLx (compile-time query validation)
- **Caching**: None for MVP (database query performance sufficient per NFR-3)
- **Read Replicas**: Planned for future, not implemented in MVP
- **Sharding**: No (single database scales to 1M+ locations/month)
- **Rationale**: SQLx provides type safety without ORM overhead, PostgreSQL handles spatial data and complex queries

### Authentication and Authorization
- **Authentication**: API key-based (SHA-256 hashed storage)
- **Authorization**: Single-level (valid API key grants full access)
- **Identity Provider**: Self-managed (api_keys table)
- **Rationale**: Simple model for family/friends use case, no user accounts needed, sufficient for trust-based groups

### Background Processing
- **Job Scheduler**: Tokio interval-based (hourly cron-like tasks)
- **Message Queue**: None (not needed for MVP)
- **Event Streaming**: None (not needed for MVP)
- **Jobs**: Location cleanup (hourly), materialized view refresh (hourly)
- **Rationale**: Simple in-process scheduler sufficient for maintenance tasks, no complex async workflows needed

### Rate Limiting
- **Strategy**: Per-API key (100 requests/minute default)
- **Implementation**: Application-level middleware with in-memory state
- **Future**: Redis-backed for multi-instance deployments
- **Rationale**: Prevents abuse, simple implementation for single-instance deployments

### Observability
- **Logging**: Structured JSON via tracing crate (configurable to pretty for dev)
- **Log Aggregation**: None (local logs for MVP)
- **Metrics**: Prometheus-compatible endpoint (`/metrics`)
- **Distributed Tracing**: None (request IDs in logs sufficient for MVP)
- **Health Checks**: Liveness (`/api/health/live`), Readiness (`/api/health/ready`), Full health (`/api/health`)
- **Alerting**: None for MVP (manual monitoring)
- **Rationale**: Prometheus industry standard, structured logs enable future aggregation, minimal ops overhead

### Deployment and Infrastructure
- **Platform Tiers**:
  - **Tier 1**: Supabase (managed PostgreSQL) + Fly.io single container
  - **Tier 2**: Fly.io multi-region + managed PostgreSQL
  - **Tier 3**: Kubernetes (self-hosted or cloud) + PostgreSQL cluster
- **Containerization**: Docker (multi-stage builds)
- **Orchestration**: Docker Compose (dev), Kubernetes (production Tier 3)
- **IaC**: Manual for MVP, Terraform for future
- **Load Balancing**: Fly.io proxy (Tier 1-2), Kubernetes Ingress (Tier 3)
- **Auto-Scaling**: Horizontal via Fly.io or Kubernetes HPA (CPU-based)
- **Rationale**: Fly.io simplifies deployment, Supabase reduces database ops burden, Kubernetes for enterprise scale

### CI/CD
- **Platform**: GitHub Actions
- **Pipeline**: Build → Test (unit + integration) → Lint (clippy, fmt) → Docker build → Deploy
- **Testing**: Unit tests, integration tests (TestServer), SQLx offline mode validation
- **Deployment Strategy**: Rolling deployment (zero-downtime)
- **Rationale**: GitHub Actions free for public repos, integrated with GitHub, Rust-friendly

### Security
- **HTTPS/TLS**: Terminated at Fly.io proxy or Kubernetes Ingress (TLS 1.3)
- **CORS**: Configurable origins via `PM__SECURITY__CORS_ORIGINS`
- **Security Headers**: X-Content-Type-Options, X-Frame-Options, Strict-Transport-Security
- **Input Sanitization**: Parameterized queries (SQLx enforces), validator crate for input validation
- **Secrets Management**: Environment variables (Fly.io secrets, Kubernetes Secrets)
- **Compliance**: GDPR (FR-24 data export/deletion)
- **Rationale**: Defense in depth, platform-managed TLS, compile-time SQL injection prevention

### Data Backup
- **Strategy**: Automated database backups
- **Implementation**: Supabase auto-backup (Tier 1), pg_dump + S3 (Tier 2-3)
- **Frequency**: Daily full backups, continuous WAL archiving
- **Retention**: 7 days minimum, 30 days for production
- **Rationale**: Disaster recovery requirement, automated to prevent human error

### Audit Logging
- **Scope**: Track all admin operations (FR-26) and privacy-sensitive operations (FR-24)
- **Data**: User (API key ID), action, timestamp, affected resources, IP address
- **Storage**: Structured logs + optional audit_log table for queryability
- **Retention**: 90 days (longer than location retention for compliance)
- **Rationale**: Security requirement, GDPR compliance, troubleshooting admin actions

## Technology Stack and Decisions

| Category | Technology | Version | Rationale |
|----------|------------|---------|-----------|
| **Language** | Rust | 1.83+ (Edition 2024) | Memory safety, performance, async/await, compile-time guarantees |
| **Web Framework** | Axum | 0.8 | Modern, tower-based, excellent async support, type-safe extractors |
| **Async Runtime** | Tokio | 1.42 | Industry standard, mature ecosystem, tracing integration |
| **HTTP Server** | Hyper | 1.5 | High-performance HTTP implementation, Axum foundation |
| **Middleware** | Tower & Tower-HTTP | 0.5 / 0.6 | Composable middleware, CORS, compression, request ID |
| **Database** | PostgreSQL | 16 | ACID compliance, spatial queries, mature, reliable |
| **Database Driver** | SQLx | 0.8 | Compile-time query validation, async, connection pooling |
| **Serialization** | Serde | 1.0 | Zero-copy deserialization, derive macros, JSON support |
| **JSON** | serde_json | 1.0 | Fast JSON parser, streaming support |
| **Validation** | Validator | 0.19 | Declarative validation, derive macros, custom validators |
| **Time/Date** | Chrono | 0.4 | Timezone-aware timestamps, Serde integration |
| **UUID** | uuid | 1.11 | V4 generation, Serde support, database compatibility |
| **Configuration** | config | 0.14 | TOML + env merging, hierarchical config |
| **Env Loading** | dotenvy | 0.15 | .env file support for development |
| **Logging** | tracing | 0.1 | Structured logging, async-aware, spans for request tracing |
| **Log Subscriber** | tracing-subscriber | 0.3 | JSON/pretty formatting, env filter, configurable output |
| **Error Handling** | thiserror | 2.0 | Domain errors with context, derive macros |
| **Error Context** | anyhow | 1.0 | Infrastructure errors, error chaining |
| **Crypto (Hashing)** | sha2 | 0.10 | API key hashing, SHA-256 implementation |
| **Crypto (Encoding)** | hex | 0.4 | Hex encoding for hashes |
| **Crypto (Random)** | rand | 0.8 | Secure random number generation |
| **Crypto (Base64)** | base64 | 0.22 | API key encoding |
| **Metrics** | metrics | 0.24 | Metrics instrumentation, counter/histogram/gauge |
| **Metrics Exporter** | metrics-exporter-prometheus | 0.16 | Prometheus text format export |
| **Testing (Async)** | tokio-test | 0.4 | Async test utilities |
| **Testing (Fixtures)** | fake | 3.0 | Test data generation, Chrono + UUID support |
| **Container Runtime** | Docker | 24+ | Multi-stage builds, slim images |
| **Deployment** | Fly.io / Kubernetes | - | Tier 1-2: Fly.io, Tier 3: K8s |
| **CI/CD** | GitHub Actions | - | Build, test, lint automation |
| **Monitoring** | Prometheus + Grafana | 2.40+ / 9.0+ | Metrics collection and visualization |

### Technology Decision Records

**Why Rust over Go/Node.js?**
- **Memory Safety**: Eliminates entire class of bugs (no null pointers, data races)
- **Performance**: Zero-cost abstractions, no garbage collection pauses
- **Async**: First-class async/await with Tokio ecosystem
- **Type Safety**: Compile-time guarantees reduce runtime errors
- **Trade-off**: Steeper learning curve, longer compile times

**Why Axum over Actix/Rocket?**
- **Modern**: Built on tower ecosystem, latest async patterns
- **Type Safety**: Extractors provide compile-time request validation
- **Performance**: Minimal overhead, excellent async performance
- **Ecosystem**: Integrates seamlessly with tower middleware
- **Trade-off**: Newer framework, smaller community than Actix

**Why SQLx over Diesel ORM?**
- **Compile-Time Checks**: SQL queries validated at compile time via macros
- **Flexibility**: Direct SQL control for complex queries (spatial, LATERAL joins)
- **Async**: Native async support without blocking
- **Simplicity**: No complex migration system, just SQL files
- **Trade-off**: Less abstraction than ORM, need to write SQL

**Why PostgreSQL over MySQL/MongoDB?**
- **ACID**: Strong consistency guarantees for location data
- **Spatial**: Future geofencing support with PostGIS extension
- **JSON**: JSONB support for flexible metadata storage
- **Performance**: Excellent for read-heavy workloads with proper indexing
- **Trade-off**: Heavier resource usage than MySQL, requires more tuning

**Why Fly.io over AWS/GCP?**
- **Simplicity**: Single command deployment, automatic HTTPS
- **Cost**: Free tier sufficient for small deployments
- **Global**: Multi-region support without complex setup
- **Rust-Friendly**: Native Rust support, fast cold starts
- **Trade-off**: Less control than raw Kubernetes, vendor lock-in

## System Architecture

### High-Level System Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        PHONE MANAGER BACKEND                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
│  │   Android    │    │   Android    │    │   Android    │               │
│  │   Client 1   │    │   Client 2   │    │   Client N   │               │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘               │
│         │                   │                   │                        │
│         │ HTTPS + X-API-Key │                   │                        │
│         └───────────────────┼───────────────────┘                        │
│                             │                                            │
│                             ▼                                            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │              LOAD BALANCER / TLS TERMINATION                      │   │
│  │                   (Fly.io Proxy / K8s Ingress)                    │   │
│  └──────────────────────────────┬───────────────────────────────────┘   │
│                                 │                                        │
│                                 ▼                                        │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                      RUST API SERVER (Axum)                       │   │
│  │  ┌────────────────────────────────────────────────────────────┐  │   │
│  │  │                    MIDDLEWARE STACK                         │  │   │
│  │  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │  │   │
│  │  │  │   CORS   │ │ Request  │ │  Trace   │ │  Timeout │      │  │   │
│  │  │  │          │ │   ID     │ │   Span   │ │  (30s)   │      │  │   │
│  │  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │  │   │
│  │  │  ┌──────────┐ ┌──────────┐ ┌──────────┐                   │  │   │
│  │  │  │   Auth   │ │   Rate   │ │ Logging  │                   │  │   │
│  │  │  │ (API Key)│ │  Limit   │ │          │                   │  │   │
│  │  │  └──────────┘ └──────────┘ └──────────┘                   │  │   │
│  │  └────────────────────────┬───────────────────────────────────┘  │   │
│  │                           │                                      │   │
│  │  ┌────────────────────────┴───────────────────────────────────┐ │   │
│  │  │                      ROUTE LAYER                            │ │   │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │ │   │
│  │  │  │   Device    │ │  Location   │ │   Health    │          │ │   │
│  │  │  │   Routes    │ │   Routes    │ │   Routes    │          │ │   │
│  │  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘          │ │   │
│  │  └─────────┼───────────────┼───────────────┼─────────────────┘ │   │
│  │            │               │               │                   │   │
│  │  ┌─────────┴───────────────┴───────────────┴─────────────────┐ │   │
│  │  │                    SERVICE LAYER                           │ │   │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │ │   │
│  │  │  │   Device    │ │  Location   │ │    Group    │          │ │   │
│  │  │  │   Service   │ │   Service   │ │   Service   │          │ │   │
│  │  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘          │ │   │
│  │  └─────────┼───────────────┼───────────────┼─────────────────┘ │   │
│  │            │               │               │                   │   │
│  │  ┌─────────┴───────────────┴───────────────┴─────────────────┐ │   │
│  │  │                  REPOSITORY LAYER                          │ │   │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │ │   │
│  │  │  │   Device    │ │  Location   │ │   API Key   │          │ │   │
│  │  │  │    Repo     │ │    Repo     │ │    Repo     │          │ │   │
│  │  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘          │ │   │
│  │  └─────────┼───────────────┼───────────────┼─────────────────┘ │   │
│  │            │               │               │                   │   │
│  │  ┌─────────┴───────────────┴───────────────┴─────────────────┐ │   │
│  │  │                 BACKGROUND JOBS                            │ │   │
│  │  │  ┌─────────────┐ ┌─────────────┐                          │ │   │
│  │  │  │  Location   │ │   Matview   │                          │ │   │
│  │  │  │   Cleanup   │ │   Refresh   │                          │ │   │
│  │  │  └─────────────┘ └─────────────┘                          │ │   │
│  │  └────────────────────────┬───────────────────────────────────┘ │   │
│  └───────────────────────────┼─────────────────────────────────────┘   │
│                              │                                          │
│                              ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    POSTGRESQL DATABASE                            │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌────────────────┐  │   │
│  │  │  devices  │ │ locations │ │ api_keys  │ │ group_member   │  │   │
│  │  │  (table)  │ │  (table)  │ │  (table)  │ │ _counts (view) │  │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └────────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                      OBSERVABILITY                                │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │   │
│  │  │ Prometheus  │ │  Structured │ │   Health    │               │   │
│  │  │  /metrics   │ │    Logs     │ │   Checks    │               │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘               │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Layered Architecture Detail

```
┌─────────────────────────────────────────────────────────────────┐
│ PRESENTATION LAYER (crates/api/src/routes/)                     │
│                                                                  │
│ • devices.rs    → Device registration, listing, deactivation    │
│ • locations.rs  → Single/batch location upload                  │
│ • health.rs     → Health checks, metrics                        │
│                                                                  │
│ Responsibilities:                                                │
│ - HTTP request/response handling                                │
│ - JSON serialization/deserialization (snake_case)               │
│ - Route parameter extraction                                    │
│ - Calls service layer                                           │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ BUSINESS LOGIC LAYER (crates/domain/src/services/)              │
│                                                                  │
│ • DeviceService    → Registration, validation, group checks     │
│ • LocationService  → Location processing, batch handling        │
│ • GroupService     → Group operations, size enforcement         │
│                                                                  │
│ Responsibilities:                                                │
│ - Business rule enforcement                                     │
│ - Cross-entity validation (group size limits)                   │
│ - Service orchestration                                         │
│ - Domain model transformations                                  │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ DATA ACCESS LAYER (crates/persistence/src/repositories/)        │
│                                                                  │
│ • DeviceRepository   → CRUD operations on devices table         │
│ • LocationRepository → Location inserts, queries, cleanup       │
│ • ApiKeyRepository   → API key validation, lookup               │
│                                                                  │
│ Responsibilities:                                                │
│ - SQLx query execution                                          │
│ - Transaction management                                        │
│ - Entity-to-domain model mapping                                │
│ - Database connection pooling                                   │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ PERSISTENCE LAYER (PostgreSQL 16)                               │
│                                                                  │
│ • Tables: devices, locations, api_keys                          │
│ • Views: devices_with_last_location, group_member_counts        │
│ • Functions: cleanup_old_locations(retention_days)              │
│ • Indexes: Optimized for query patterns                         │
└─────────────────────────────────────────────────────────────────┘
```

### Request Flow Diagram

```
┌──────────────┐
│ Mobile Client│
└──────┬───────┘
       │ POST /api/v1/locations
       │ Headers: X-API-Key, X-Request-ID
       │ Body: {"device_id": "...", "latitude": 48.14, ...}
       │
       ▼
┌──────────────────────────────────────────────┐
│ Middleware Stack (Applied in Order)         │
├──────────────────────────────────────────────┤
│ 1. CORS → Validate origin                   │
│ 2. Request ID → Generate or extract         │
│ 3. Trace Span → Create tracing context      │
│ 4. Timeout → Start 30s timer                │
│ 5. Auth → Validate API key, get key_id      │
│ 6. Rate Limit → Check requests/min          │
│ 7. Logging → Log request start              │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ Route Handler (locations.rs)                │
├──────────────────────────────────────────────┤
│ 1. Deserialize JSON to LocationPayload      │
│ 2. Validate via validator crate             │
│ 3. Extract authenticated API key from state │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ Service Layer (LocationService)             │
├──────────────────────────────────────────────┤
│ 1. Verify device exists                     │
│ 2. Convert timestamp (ms → DateTime)        │
│ 3. Apply business rules                     │
│ 4. Call repository                          │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ Repository Layer (LocationRepository)       │
├──────────────────────────────────────────────┤
│ 1. Get connection from pool                 │
│ 2. Execute INSERT via SQLx                  │
│ 3. Map result to domain model               │
│ 4. Update device.last_seen_at (async)       │
└──────┬───────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────┐
│ PostgreSQL Database                         │
├──────────────────────────────────────────────┤
│ 1. Validate constraints                     │
│ 2. Insert into locations table              │
│ 3. Return row ID                            │
└──────┬───────────────────────────────────────┘
       │
       ▼ (Response flows back up)
┌──────────────────────────────────────────────┐
│ Response                                     │
├──────────────────────────────────────────────┤
│ 200 OK                                       │
│ {"success": true, "processed_count": 1}     │
│                                              │
│ Logged: method, path, status, duration_ms   │
│ Metrics: request counter++, latency recorded│
└──────────────────────────────────────────────┘
```

### Data Flow Patterns

#### Device Registration Flow
```
Mobile Client
    │
    ├─ POST /api/v1/devices/register
    │  Body: {device_id, display_name, group_id, fcm_token}
    │
    ▼
Middleware (Auth, Validation)
    │
    ▼
DeviceService
    ├─ Check group size via DeviceRepository.count_active_by_group()
    ├─ If >= 20 → Return 409 Conflict
    │
    ▼
DeviceRepository
    ├─ UPSERT device (INSERT ... ON CONFLICT DO UPDATE)
    ├─ Update last_seen_at, updated_at
    │
    ▼
PostgreSQL devices table
    │
    ▼ Return
{device_id, display_name, group_id, created_at, updated_at}
```

#### Batch Location Upload Flow
```
Mobile Client
    │
    ├─ POST /api/v1/locations/batch
    │  Body: {device_id, locations: [50 items]}
    │
    ▼
Middleware (Auth, Rate Limit, Timeout=30s)
    │
    ▼
LocationService
    ├─ Verify device exists
    ├─ Validate all 50 locations
    ├─ Check idempotency key (if provided)
    │
    ▼
LocationRepository.insert_batch()
    ├─ BEGIN TRANSACTION (atomic)
    ├─ INSERT INTO locations VALUES (...), (...), ... (batch)
    ├─ COMMIT or ROLLBACK
    │
    ▼
PostgreSQL locations table
    ├─ 50 rows inserted atomically
    │
    ▼ Return
{success: true, processed_count: 50}
```

#### Group Device Listing Flow
```
Mobile Client
    │
    ├─ GET /api/v1/devices?group_id=family-123
    │
    ▼
Middleware (Auth)
    │
    ▼
DeviceService.get_group_devices(group_id)
    │
    ▼
DeviceRepository
    ├─ Query devices_with_last_location view
    ├─ Filter: group_id = ? AND active = true
    ├─ LATERAL JOIN for last location
    │
    ▼
PostgreSQL
    ├─ Return: device + last location data
    │
    ▼ Map to domain models
{
  "devices": [
    {
      "device_id": "...",
      "display_name": "...",
      "last_location": {...},
      "last_seen_at": "..."
    }
  ]
}
```

### Background Job Architecture

```
┌─────────────────────────────────────────────┐
│ Main Tokio Runtime                          │
│                                              │
│  ┌────────────────────────────────────────┐ │
│  │ HTTP Server (Axum)                     │ │
│  │ - Handles requests                     │ │
│  └────────────────────────────────────────┘ │
│                                              │
│  ┌────────────────────────────────────────┐ │
│  │ Background Job Scheduler               │ │
│  │                                        │ │
│  │  ┌──────────────────────────────────┐ │ │
│  │  │ Location Cleanup Job             │ │ │
│  │  │ - Interval: Every 1 hour         │ │ │
│  │  │ - Action: DELETE old locations   │ │ │
│  │  │ - Query: WHERE created_at < -30d │ │ │
│  │  └──────────────────────────────────┘ │ │
│  │                                        │ │
│  │  ┌──────────────────────────────────┐ │ │
│  │  │ Materialized View Refresh Job    │ │ │
│  │  │ - Interval: Every 1 hour         │ │ │
│  │  │ - Action: REFRESH MATERIALIZED   │ │ │
│  │  │   VIEW group_member_counts       │ │ │
│  │  └──────────────────────────────────┘ │ │
│  └────────────────────────────────────────┘ │
│                                              │
│  Graceful Shutdown:                         │
│  1. Stop accepting new requests             │
│  2. Finish in-flight requests               │
│  3. Cancel background jobs gracefully       │
│  4. Close database connections              │
└─────────────────────────────────────────────┘
```

### Deployment Architecture

#### Tier 1: Minimal (Supabase + Fly.io)
```
┌────────────────┐
│ Mobile Clients │
└───────┬────────┘
        │ HTTPS
        ▼
┌────────────────────────┐
│ Fly.io                 │
│ ┌────────────────────┐ │
│ │ Single Container   │ │
│ │ (phone-manager)    │ │
│ │ - 256MB RAM        │ │
│ │ - 1 CPU            │ │
│ └─────────┬──────────┘ │
└───────────┼────────────┘
            │
            ▼
┌────────────────────────┐
│ Supabase PostgreSQL    │
│ - Managed backups      │
│ - Connection pooler    │
└────────────────────────┘
```

#### Tier 2-3: Production (Fly.io/Kubernetes)
```
┌────────────────┐
│ Mobile Clients │
└───────┬────────┘
        │ HTTPS
        ▼
┌──────────────────────────────┐
│ Load Balancer + TLS          │
│ (Fly.io Proxy / K8s Ingress) │
└───────┬──────────────────────┘
        │
        ▼
┌──────────────────────────────┐
│ API Server (3+ replicas)     │
│ ┌──────┐ ┌──────┐ ┌──────┐  │
│ │ Pod 1│ │ Pod 2│ │ Pod 3│  │
│ └───┬──┘ └───┬──┘ └───┬──┘  │
└─────┼────────┼────────┼──────┘
      └────────┼────────┘
               │
               ▼
┌──────────────────────────────┐
│ PostgreSQL Cluster           │
│ - Primary + Read Replicas    │
│ - Automated backups          │
│ - Point-in-time recovery     │
└──────────────────────────────┘
```

### Scalability Strategy

**Horizontal Scaling Approach:**

1. **Stateless Design**: No session state in API servers
2. **Database Connection Pooling**: 20-100 connections per instance
3. **Read Replica Support** (future): Route read queries to replicas
4. **Caching Strategy** (future): Redis for rate limiting, session cache

**Performance Optimization Points:**

| Component | Optimization | Target |
|-----------|-------------|--------|
| API Response | Async I/O, zero-copy serialization | <200ms p95 |
| Database Queries | Indexes on all foreign keys, covering indexes | <100ms p95 |
| Location Inserts | Batch inserts, prepared statements | <10ms single, <100ms batch |
| Group Queries | Materialized view, LATERAL join optimization | <50ms |
| Background Jobs | Batched deletes (10K rows), non-blocking | <5min |

**Scale Limits:**

| Metric | MVP Target | Future Capacity |
|--------|------------|-----------------|
| Concurrent Connections | 10,000 | 100,000+ (horizontal scaling) |
| Requests/Second | 1,000 | 10,000+ (load balancing) |
| Locations/Month | 1M | 100M+ (partitioning) |
| Devices Total | 10,000 | 1M+ (sharding) |
| Groups | 1,000 | 100K+ (indexing) |

## Data Architecture

### Database Schema

```sql
┌─────────────────────────────────────────────────────────────┐
│ devices                                                      │
├─────────────────────────────────────────────────────────────┤
│ id              BIGSERIAL PRIMARY KEY                        │
│ device_id       UUID NOT NULL UNIQUE                         │
│ display_name    VARCHAR(50) NOT NULL                         │
│ group_id        VARCHAR(50) NOT NULL                         │
│ platform        VARCHAR(20) NOT NULL DEFAULT 'android'       │
│ fcm_token       TEXT                                         │
│ active          BOOLEAN NOT NULL DEFAULT true                │
│ last_seen_at    TIMESTAMPTZ                                  │
│ created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()           │
│ updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()           │
├─────────────────────────────────────────────────────────────┤
│ Indexes:                                                     │
│ - idx_devices_device_id (device_id) UNIQUE                  │
│ - idx_devices_group_id (group_id)                           │
│ - idx_devices_fcm_token (fcm_token) WHERE NOT NULL          │
│ - idx_devices_active (group_id, active)                     │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ 1:N
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ locations                                                    │
├─────────────────────────────────────────────────────────────┤
│ id              BIGSERIAL PRIMARY KEY                        │
│ device_id       UUID NOT NULL REFERENCES devices(device_id) │
│                 ON DELETE CASCADE                            │
│ latitude        DOUBLE PRECISION NOT NULL                    │
│ longitude       DOUBLE PRECISION NOT NULL                    │
│ accuracy        REAL NOT NULL                                │
│ altitude        DOUBLE PRECISION                             │
│ bearing         REAL                                         │
│ speed           REAL                                         │
│ provider        VARCHAR(50)                                  │
│ battery_level   SMALLINT                                     │
│ network_type    VARCHAR(50)                                  │
│ captured_at     TIMESTAMPTZ NOT NULL                         │
│ created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()           │
├─────────────────────────────────────────────────────────────┤
│ Constraints:                                                 │
│ - chk_latitude: latitude BETWEEN -90 AND 90                 │
│ - chk_longitude: longitude BETWEEN -180 AND 180             │
│ - chk_accuracy: accuracy >= 0                               │
│ - chk_bearing: bearing BETWEEN 0 AND 360 (if not null)      │
│ - chk_speed: speed >= 0 (if not null)                       │
│ - chk_battery: battery_level BETWEEN 0 AND 100 (if not null)│
├─────────────────────────────────────────────────────────────┤
│ Indexes:                                                     │
│ - idx_locations_device_captured (device_id, captured_at DESC)│
│ - idx_locations_created_at (created_at) [for cleanup]       │
│ - idx_locations_recent (device_id, captured_at DESC)        │
│   WHERE captured_at > NOW() - INTERVAL '24 hours'           │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ api_keys                                                     │
├─────────────────────────────────────────────────────────────┤
│ id              BIGSERIAL PRIMARY KEY                        │
│ key_hash        VARCHAR(128) NOT NULL UNIQUE                 │
│ key_prefix      VARCHAR(8) NOT NULL                          │
│ name            VARCHAR(100) NOT NULL                        │
│ is_active       BOOLEAN NOT NULL DEFAULT TRUE                │
│ is_admin        BOOLEAN NOT NULL DEFAULT FALSE               │
│ last_used_at    TIMESTAMPTZ                                  │
│ created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()           │
│ expires_at      TIMESTAMPTZ                                  │
├─────────────────────────────────────────────────────────────┤
│ Indexes:                                                     │
│ - idx_api_keys_hash (key_hash) WHERE is_active = TRUE       │
│ - idx_api_keys_prefix (key_prefix)                          │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ audit_log (optional - for FR-24, FR-26)                     │
├─────────────────────────────────────────────────────────────┤
│ id              BIGSERIAL PRIMARY KEY                        │
│ api_key_id      BIGINT REFERENCES api_keys(id)              │
│ action          VARCHAR(100) NOT NULL                        │
│ resource_type   VARCHAR(50) NOT NULL                         │
│ resource_id     TEXT                                         │
│ metadata        JSONB                                        │
│ ip_address      INET                                         │
│ created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()           │
├─────────────────────────────────────────────────────────────┤
│ Indexes:                                                     │
│ - idx_audit_api_key (api_key_id, created_at DESC)           │
│ - idx_audit_resource (resource_type, resource_id)           │
│ - idx_audit_created (created_at DESC)                       │
└─────────────────────────────────────────────────────────────┘
```

### Materialized Views

```sql
CREATE MATERIALIZED VIEW group_member_counts AS
SELECT
    group_id,
    COUNT(*) FILTER (WHERE active = true) as active_count,
    COUNT(*) as total_count,
    MAX(last_seen_at) as last_activity
FROM devices
GROUP BY group_id;

CREATE UNIQUE INDEX idx_group_member_counts ON group_member_counts(group_id);
```

**Refresh Strategy**: Hourly via background job, `REFRESH MATERIALIZED VIEW CONCURRENTLY`

### Domain Models

```rust
// Core domain entities (crates/domain/src/models/)

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

pub struct ApiKey {
    pub id: i64,
    pub key_hash: String,
    pub key_prefix: String,
    pub name: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### Data Flow and Relationships

```
Group (logical grouping via group_id string)
    │
    │ 1:N (max 20 active devices)
    ▼
Device (identity + metadata)
    │
    │ 1:N (locations over time)
    ▼
Location (GPS coordinates + metadata)
    │
    │ Retention: 30 days
    ▼
[Auto-deleted by cleanup job]

API Key
    │
    │ 1:N (authenticates all requests)
    ▼
Audit Log Entry (tracks admin operations)
```

### Data Retention and Lifecycle

| Data Type | Retention | Cleanup Method | Frequency |
|-----------|-----------|----------------|-----------|
| Location records | 30 days | Background job DELETE | Hourly |
| Device records | Indefinite | Manual/admin only (FR-26) | N/A |
| API keys | Until expired/deactivated | Manual | N/A |
| Audit logs | 90 days | Background job DELETE | Daily |
| Materialized views | Refreshed hourly | REFRESH CONCURRENTLY | Hourly |

### Database Performance Strategy

**Connection Pooling**:
- Min connections: 5 (keep-alive)
- Max connections: 20 (per instance)
- Idle timeout: 600 seconds (10 minutes)
- Connect timeout: 10 seconds

**Query Optimization**:
- All foreign keys indexed
- Covering indexes for common queries
- Partial indexes for filtered queries (e.g., active devices, recent locations)
- EXPLAIN ANALYZE validation during development

**Transaction Strategy**:
- Single-row operations: Auto-commit
- Batch uploads: Explicit transaction with ROLLBACK on any validation failure
- Admin operations: Transaction for data consistency

### Data Validation Layers

**Defense in Depth**:

1. **Application Layer** (validator crate):
   - Latitude: -90.0 to 90.0
   - Longitude: -180.0 to 180.0
   - Accuracy: >= 0.0
   - Display name: 2-50 chars
   - Group ID: 2-50 chars, alphanumeric + hyphens/underscores

2. **Database Layer** (CHECK constraints):
   - Constraints enforce same rules as application
   - Prevents invalid data if application validation bypassed
   - Last line of defense

3. **Type System** (Rust):
   - f64 for coordinates (proper precision)
   - Uuid for device identifiers (prevents typos)
   - DateTime<Utc> for timestamps (timezone-aware)

### Data Privacy Architecture

**GDPR Compliance (FR-24)**:

1. **Data Export**:
   - Endpoint: `GET /api/v1/devices/:device_id/data-export`
   - Returns: All device data + all location records (JSON)
   - Streaming response for large datasets

2. **Data Deletion**:
   - Endpoint: `DELETE /api/v1/devices/:device_id/data`
   - Cascades: All associated locations via ON DELETE CASCADE
   - Audit logged: Who deleted, when, what device

3. **Retention Policy**:
   - Automatic deletion after 30 days
   - No indefinite storage
   - User-visible in API responses

## API Design

### REST API Specification

**Base URL**: `https://api.phonemanager.example.com/api/v1`

**API Versioning Strategy** (FR-25):
- URL-based versioning: `/api/v1/`, `/api/v2/` (future)
- Version in path for clarity and client routing
- Old versions supported for 6 months after new version release
- Breaking changes require new version

### Authentication

**Method**: API Key via HTTP header

```http
X-API-Key: pm_aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890
```

**Key Format**:
- Prefix: `pm_` (identifies Phone Manager keys)
- Length: 48 characters total (3 char prefix + 45 char random)
- Generation: Base64-encoded cryptographically secure random bytes
- Storage: SHA-256 hash only, never plaintext

### Request/Response Format

**Content Type**: `application/json`

**Field Naming**: **snake_case** (aligned with Rust conventions)

**Example Request**:
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "Martin's Phone",
  "group_id": "family-group-123",
  "platform": "android",
  "fcm_token": "firebase-token-here"
}
```

**Example Response**:
```json
{
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "Martin's Phone",
  "group_id": "family-group-123",
  "created_at": "2025-11-25T10:30:00Z",
  "updated_at": "2025-11-25T10:30:00Z"
}
```

### Error Response Format (FR-21)

**Structure**:
```json
{
  "error": "error_code",
  "message": "Human-readable error message",
  "details": [
    {
      "field": "latitude",
      "message": "Latitude must be between -90 and 90"
    }
  ]
}
```

**HTTP Status Codes**:
- `200 OK` - Success
- `201 Created` - Resource created (future use)
- `204 No Content` - Success with no response body (deletes)
- `400 Bad Request` - Validation error, malformed request
- `401 Unauthorized` - Missing or invalid API key
- `404 Not Found` - Resource doesn't exist
- `409 Conflict` - Business rule violation (e.g., group full)
- `413 Payload Too Large` - Request exceeds 1MB
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Unexpected server error
- `503 Service Unavailable` - Database unavailable

**Error Code Conventions**:
- `validation_error` - Input validation failed
- `unauthorized` - Authentication failed
- `not_found` - Resource doesn't exist
- `conflict` - Business rule violation
- `rate_limit_exceeded` - Too many requests
- `service_unavailable` - Temporary failure
- `internal_error` - Unexpected error

### API Endpoints

#### Device Management

**POST /api/v1/devices/register**
- Register new device or update existing device
- Auth: Required
- Rate Limit: 100/min per API key
- Request Body: `DeviceRegistrationRequest`
- Response: 200 with `DeviceRegistrationResponse` or 409 if group full

**GET /api/v1/devices**
- List all active devices in a group
- Auth: Required
- Query Params: `group_id` (required)
- Rate Limit: 100/min per API key
- Response: 200 with `DevicesResponse` (includes last_location)

**DELETE /api/v1/devices/:device_id**
- Soft delete device (set active=false)
- Auth: Required
- Rate Limit: 100/min per API key
- Response: 204 No Content

#### Location Tracking

**POST /api/v1/locations**
- Upload single location point
- Auth: Required
- Rate Limit: 100/min per API key
- Request Body: `LocationPayload`
- Headers: Optional `Idempotency-Key` for retry safety (FR-22)
- Response: 200 with `LocationUploadResponse`

**POST /api/v1/locations/batch**
- Upload batch of locations (max 50)
- Auth: Required
- Rate Limit: 100/min per API key
- Request Body: `LocationBatchPayload`
- Headers: Optional `Idempotency-Key`
- Timeout: 30 seconds
- Response: 200 with `LocationUploadResponse` (processed_count)

#### Admin Operations

**DELETE /api/v1/admin/devices/inactive**
- Bulk delete inactive devices
- Auth: Required (admin API key only)
- Query Params: `older_than_days` (required)
- Rate Limit: 1000/min for admin keys
- Response: 200 with count of deleted devices

**GET /api/v1/devices/:device_id/data-export**
- Export all device data for GDPR compliance (FR-24)
- Auth: Required
- Rate Limit: 10/hour per API key (expensive operation)
- Response: 200 with streaming JSON (device + all locations)

**DELETE /api/v1/devices/:device_id/data**
- Complete data deletion (device + all locations)
- Auth: Required
- Rate Limit: 10/hour per API key
- Audit: Logged to audit_log table
- Response: 204 No Content

#### Health and Metrics

**GET /api/health**
- Full health check with database latency
- Auth: Not required
- Response: 200 or 503 with `HealthResponse`

**GET /api/health/live**
- Kubernetes liveness probe
- Auth: Not required
- Response: 200 "OK" (process alive)

**GET /api/health/ready**
- Kubernetes readiness probe (database connectivity)
- Auth: Not required
- Response: 200 "OK" (can accept traffic)

**GET /metrics**
- Prometheus metrics export
- Auth: Not required (may add basic auth in production)
- Response: 200 with Prometheus text format

### Request Headers

**Standard Headers**:
- `Content-Type: application/json` (required for POST/PUT)
- `X-API-Key: <key>` (required for authenticated endpoints)

**Optional Headers**:
- `X-Request-ID: <uuid>` (for request tracing, auto-generated if missing)
- `Idempotency-Key: <uuid>` (for location uploads to prevent duplicates)

**Response Headers**:
- `X-Request-ID: <uuid>` (echoed back for correlation)
- `Retry-After: <seconds>` (on 429 rate limit)
- Security headers (X-Content-Type-Options, X-Frame-Options, Strict-Transport-Security)

### Rate Limiting Behavior (FR-16)

**Strategy**: Sliding window per API key

**Default Limits**:
- Standard endpoints: 100 requests/minute
- Admin endpoints: 1000 requests/minute
- Data export: 10 requests/hour
- Configurable via `PM__SECURITY__RATE_LIMIT_PER_MINUTE`

**Response on Limit Exceeded**:
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 45
Content-Type: application/json

{
  "error": "rate_limit_exceeded",
  "message": "Rate limit of 100 requests/minute exceeded",
  "retry_after": 45
}
```

### Idempotency Strategy (FR-22)

**Endpoints Supporting Idempotency**:
- `POST /api/v1/locations`
- `POST /api/v1/locations/batch`

**Mechanism**:
```http
POST /api/v1/locations
Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000
```

**Behavior**:
- First request: Process normally, cache response for 24 hours
- Duplicate request (same key): Return cached response (same status code + body)
- Prevents duplicate location records from network retries
- Keys expire after 24 hours

**Implementation**:
- Store `idempotency_key` hash + response in database or Redis
- `ON CONFLICT (idempotency_key_hash) DO NOTHING` for simple deduplication

### API Contract Validation

**Request Validation**:
- JSON schema validation via `serde` deserialization
- Business rule validation via `validator` crate
- Type-safe via Rust struct definitions

**Response Guarantees**:
- All timestamps in ISO 8601 format (UTC)
- All UUIDs in standard format (8-4-4-4-12)
- All numeric IDs as integers (never strings)
- Consistent error format across all endpoints

## Cross-Cutting Concerns

### Logging and Tracing

**Strategy**: Structured logging with request tracing spans

**Implementation**:
- Library: `tracing` + `tracing-subscriber`
- Format: JSON in production (`PM__LOGGING__FORMAT=json`), pretty in development
- Log Level: Configurable via `PM__LOGGING__LEVEL` (trace, debug, info, warn, error)

**Request Tracing**:
```rust
// Every request gets a span with request_id
tracing::info_span!(
    "http_request",
    method = %req.method(),
    path = %req.uri().path(),
    request_id = %request_id,
)
```

**Log Structure**:
```json
{
  "timestamp": "2025-11-25T10:30:00.123Z",
  "level": "INFO",
  "target": "phone_manager_api::routes::locations",
  "message": "Location uploaded successfully",
  "fields": {
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "device_id": "660e8400-e29b-41d4-a716-446655440001",
    "method": "POST",
    "path": "/api/v1/locations",
    "status": 200,
    "duration_ms": 45
  }
}
```

**What Gets Logged**:
- All HTTP requests (method, path, status, duration, request_id)
- Database queries at debug level (query, duration)
- Authentication events (key validated, rate limit exceeded)
- Background job execution (start, end, rows affected)
- Errors with full context and stack traces
- **Never logged**: API keys, passwords, location data in plain text

### Error Handling

**Error Type Hierarchy**:

```rust
// Domain errors (crates/domain/src/errors.rs)
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Device not found: {0}")]
    DeviceNotFound(Uuid),

    #[error("Group is full (max 20 devices)")]
    GroupFull,

    #[error("Invalid group ID: {0}")]
    InvalidGroupId(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),
}

// API errors (crates/api/src/error.rs)
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded { retry_after: u64 },

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),

    #[error("Domain error")]
    Domain(#[from] DomainError),
}
```

**Error Propagation**:
- Repository → Domain errors (thiserror)
- Service → Domain errors
- Routes → API errors (maps domain errors to HTTP)
- Infrastructure → anyhow::Error

**Error Response Mapping**:
```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg),
            ApiError::RateLimitExceeded { retry_after } => {
                // Include Retry-After header
                (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded", format!("..."))
            },
            ApiError::Domain(DomainError::DeviceNotFound(_)) => {
                (StatusCode::NOT_FOUND, "not_found", "Device not found")
            },
            ApiError::Domain(DomainError::GroupFull) => {
                (StatusCode::CONFLICT, "conflict", "Group has reached maximum device limit (20)")
            },
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "An unexpected error occurred"),
        };

        // Construct JSON response with ErrorResponse struct
    }
}
```

### Configuration Management

**Configuration Sources** (priority order):
1. Environment variables (`PM__*`)
2. `config/local.toml` (git-ignored, developer overrides)
3. `config/default.toml` (committed defaults)

**Configuration Structure**:
```rust
pub struct Config {
    pub server: ServerConfig,      // host, port, timeouts
    pub database: DatabaseConfig,  // URL, pool settings
    pub logging: LoggingConfig,    // level, format
    pub security: SecurityConfig,  // CORS, rate limits
    pub limits: LimitsConfig,      // max devices, batch size, retention
}
```

**Environment Variable Pattern**:
```bash
PM__DATABASE__URL=postgres://...
PM__SERVER__PORT=8080
PM__LIMITS__MAX_DEVICES_PER_GROUP=20
```

**Validation**: Config validation at startup, fail-fast on missing required values

### Middleware Stack

**Execution Order** (tower layers, applied bottom-to-top):

1. **CORS** (`tower-http::cors`)
   - Validates origin against `PM__SECURITY__CORS_ORIGINS`
   - Allows credentials for API key header
   - Preflight request handling

2. **Request ID** (`tower-http::request_id`)
   - Generates UUID if `X-Request-ID` not provided
   - Propagates through all layers
   - Included in response headers

3. **Trace Span** (`tower-http::trace`)
   - Creates tracing span for request
   - Logs request start, end, duration
   - Integrates with structured logging

4. **Timeout** (`tower::timeout`)
   - 30-second timeout for all requests
   - Returns 408 Request Timeout
   - Prevents hanging requests

5. **Compression** (`tower-http::compression`)
   - Gzip response compression
   - Saves bandwidth for large responses
   - Automatic based on Accept-Encoding header

6. **Auth Extractor** (custom)
   - Validates API key from `X-API-Key` header
   - Queries database for key validation
   - Injects authenticated context into request

7. **Rate Limiting** (custom, using `governor` crate)
   - Per-API key sliding window
   - Returns 429 with Retry-After when exceeded
   - In-memory state (Redis for multi-instance future)

8. **Request Logging** (`tower-http::trace`)
   - Logs completed request with all metadata
   - Includes status, duration, errors

### Security

**Defense in Depth Layers**:

1. **Network**: TLS 1.3 at load balancer (Fly.io/K8s Ingress)
2. **Authentication**: API key required for all endpoints (except health)
3. **Input Validation**: validator crate + database constraints
4. **SQL Injection**: Prevented via SQLx parameterized queries
5. **Rate Limiting**: Per-API key to prevent abuse
6. **CORS**: Configurable origin restrictions

**Security Headers** (tower-http):
```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

**Secret Management**:
- Development: `.env` file (git-ignored)
- Fly.io: `fly secrets set PM__DATABASE__URL=...`
- Kubernetes: Kubernetes Secrets mounted as env vars
- Never commit secrets to repository

### Observability

**Metrics Exposed** (Prometheus format at `/metrics`):

```
# Request metrics
http_requests_total{method, path, status}          # Counter
http_request_duration_seconds{path}                # Histogram (p50, p90, p95, p99)

# Database metrics
database_connections_active                         # Gauge
database_connections_idle                          # Gauge
database_query_duration_seconds{query_type}        # Histogram

# Business metrics
locations_uploaded_total                           # Counter
devices_registered_total                           # Counter
active_devices_count{group_id}                     # Gauge (from matview)

# Background jobs
background_job_duration_seconds{job_name}          # Histogram
background_job_last_success_timestamp{job_name}    # Gauge
locations_deleted_total{job="cleanup"}             # Counter
```

**Histogram Buckets**: [0.001, 0.005, 0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0] seconds

**Health Check Integration**:
- Liveness: Returns 200 if process running
- Readiness: Returns 200 if database accessible (simple SELECT 1 query)
- Full health: Returns detailed status with database latency measurement

### Background Jobs

**Job Scheduler Implementation**:

```rust
// crates/api/src/jobs/mod.rs

pub struct JobScheduler {
    pool: PgPool,
    config: Config,
}

impl JobScheduler {
    pub async fn start(pool: PgPool, config: Config) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour
            let mut refresh_interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour

            loop {
                tokio::select! {
                    _ = cleanup_interval.tick() => {
                        Self::run_location_cleanup(&pool, &config).await;
                    }
                    _ = refresh_interval.tick() => {
                        Self::refresh_materialized_views(&pool).await;
                    }
                }
            }
        })
    }
}
```

**Graceful Shutdown**:
1. Signal handler catches SIGTERM/SIGINT
2. Stop accepting new HTTP requests
3. Wait for in-flight requests (max 30s timeout)
4. Cancel background job tasks
5. Close database connection pool
6. Exit cleanly

### Dependency Injection

**Application State Pattern**:

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub rate_limiter: Arc<RateLimiter>,
}

// Injected into all handlers via Axum State extractor
pub async fn register_device(
    State(state): State<AppState>,
    ApiKeyAuth { api_key_id, .. }: ApiKeyAuth,
    Json(payload): Json<DeviceRegistrationRequest>,
) -> Result<Json<DeviceRegistrationResponse>, ApiError> {
    // Access pool, config, rate_limiter from state
}
```

**Benefits**:
- Testable (mock state in tests)
- Shared resources (single pool, config)
- Type-safe injection via Axum extractors

## Component and Integration Overview

### Module Communication Patterns

#### Routes → Services → Repositories Flow

```rust
// Example: Device Registration Flow

// 1. Route Layer (crates/api/src/routes/devices.rs)
pub async fn register_device(
    State(state): State<AppState>,
    ApiKeyAuth { api_key_id, .. }: ApiKeyAuth,
    Json(payload): Json<DeviceRegistrationRequest>,
) -> Result<Json<DeviceRegistrationResponse>, ApiError> {
    // Validate payload (validator crate)
    payload.validate()?;

    // Call service layer
    let device_service = DeviceService::new(&state.pool);
    let device = device_service.register(payload).await?;

    // Map to response
    Ok(Json(device.into()))
}

// 2. Service Layer (crates/domain/src/services/device.rs)
impl DeviceService {
    pub async fn register(&self, req: DeviceRegistrationRequest) -> Result<Device, DomainError> {
        // Business logic: Check group size
        let group_size = self.repo.count_active_by_group(&req.group_id).await?;
        if group_size >= 20 {
            return Err(DomainError::GroupFull);
        }

        // Call repository
        self.repo.upsert_device(req).await
    }
}

// 3. Repository Layer (crates/persistence/src/repositories/device.rs)
impl DeviceRepository {
    pub async fn upsert_device(&self, req: DeviceRegistrationRequest) -> Result<Device, DomainError> {
        let entity = sqlx::query_as!(
            DeviceEntity,
            r#"
            INSERT INTO devices (device_id, display_name, group_id, platform, fcm_token)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (device_id) DO UPDATE
            SET display_name = $2, fcm_token = $5, updated_at = NOW()
            RETURNING *
            "#,
            req.device_id, req.display_name, req.group_id, req.platform, req.fcm_token
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(entity.into()) // Map entity to domain model
    }
}
```

### Crate Interaction Matrix

| From Crate | To Crate | Interaction Type | Purpose |
|------------|----------|------------------|---------|
| api | domain | Function calls | Business logic invocation |
| api | persistence | Indirect (via domain) | Data access (should go through services) |
| api | shared | Function calls | Utilities (hashing, validation) |
| domain | shared | Function calls | Common utilities |
| domain | persistence | **NOT ALLOWED** | Maintains separation of concerns |
| persistence | domain | Type imports only | Domain model definitions |
| persistence | shared | Function calls | Utilities |

**Architecture Rule**: Routes should **never** call repositories directly. Always go through service layer for business logic.

### State Management

**Application State** (shared across all requests):
```rust
pub struct AppState {
    pub pool: PgPool,                    // Database connection pool
    pub config: Arc<Config>,             // Immutable configuration
    pub rate_limiter: Arc<RateLimiter>,  // Shared rate limit state
}
```

**Request-Scoped State**:
- `ApiKeyAuth` - Authenticated API key context (injected by extractor)
- `RequestId` - Unique request identifier (injected by middleware)
- Tracing span - Request tracing context

**No Session State**: API is fully stateless, scales horizontally without sticky sessions

### External Integrations

#### PostgreSQL Database
- **Connection**: Connection pool (5-20 connections)
- **Protocol**: PostgreSQL wire protocol
- **Driver**: SQLx async driver
- **Pooling Strategy**: Maintain min connections, scale to max on demand
- **Health Check**: Simple `SELECT 1` query every 30 seconds

#### Prometheus Monitoring
- **Integration**: Scrape-based (Prometheus pulls from `/metrics`)
- **Frequency**: Every 15 seconds (configurable)
- **Format**: Prometheus text exposition format
- **No Authentication**: Public endpoint (consider basic auth in production)

#### Future: Firebase Cloud Messaging (Out of Scope)
- **Purpose**: Push notifications for location updates
- **Integration**: HTTPS API calls to FCM
- **Token Storage**: `fcm_token` column in devices table (already present)
- **Implementation**: Post-MVP (Phase 4)

### Component Testing Strategy

**Unit Tests** (per crate):
- **domain**: Pure business logic, no database
- **persistence**: Repository tests with test database
- **shared**: Utility function tests

**Integration Tests** (`tests/` directory):
- Full API tests using `TestServer`
- Test database per test (isolated)
- End-to-end scenarios (register → upload → query)

**Test Database Strategy**:
```rust
// Create unique database per test
let test_db_name = format!("test_{}", Uuid::new_v4());
// Run migrations
// Execute test
// Drop database
```

## Architecture Decision Records

### ADR-001: snake_case for JSON API

**Status**: Accepted
**Date**: 2025-11-25
**Context**: Need to choose JSON field naming convention for REST API

**Decision**: Use **snake_case** for all JSON field names (e.g., `device_id`, `display_name`)

**Rationale**:
- Aligns with Rust naming conventions (consistency across codebase)
- Serde defaults to struct field names (no rename needed)
- Reduces cognitive load for Rust developers
- Industry examples: GitHub API, Stripe API use snake_case

**Consequences**:
- Mobile client must use snake_case in API calls
- Different from some JavaScript conventions (camelCase)
- Simpler Serde configuration (no `rename_all` attribute needed)

**Alternatives Considered**:
- camelCase: Common in JavaScript ecosystems but inconsistent with Rust
- PascalCase: Not standard for JSON APIs
- kebab-case: Not valid in most programming languages

---

### ADR-002: Modular Monolith over Microservices

**Status**: Accepted
**Date**: 2025-11-25
**Context**: Need to choose service architecture for backend API

**Decision**: Single Rust binary with workspace crates (modular monolith)

**Rationale**:
- Small team (1-3 developers) doesn't need microservices complexity
- In-process communication eliminates network latency
- Single deployment simplifies operations
- Crate boundaries provide modularity without distribution
- Can extract to microservices later if needed (unlikely given scale)

**Consequences**:
- All components deployed together (no independent scaling)
- Shared database connection pool
- Single point of failure (mitigated by horizontal scaling)
- Faster development velocity

**Performance Impact**:
- Eliminates ~10-50ms network overhead per internal call
- Shared memory enables zero-copy optimizations

---

### ADR-003: SQLx over Diesel ORM

**Status**: Accepted
**Date**: 2025-11-25
**Context**: Need database access layer for PostgreSQL

**Decision**: Use SQLx with compile-time query validation

**Rationale**:
- Compile-time SQL validation prevents runtime query errors
- Direct SQL control needed for complex queries (LATERAL joins, materialized views)
- Native async support without blocking
- Simpler than Diesel for this use case (no complex relations)

**Consequences**:
- Must write SQL (not generated by ORM)
- Requires `cargo sqlx prepare` for offline mode (CI builds)
- Better performance for complex queries
- Steeper learning curve for non-SQL developers

**Migration Strategy**:
- SQL files in `crates/persistence/src/migrations/`
- Applied via `sqlx::migrate!()` macro at startup
- Versioned and sequential

---

### ADR-004: API Key Authentication (No User Accounts)

**Status**: Accepted
**Date**: 2025-11-25
**Context**: Need authentication mechanism for API

**Decision**: API key-based authentication with SHA-256 hashed storage

**Rationale**:
- Simpler than OAuth/JWT for family/friends use case
- No user account system needed (trust-based groups)
- Sufficient security for non-public API
- Easy to implement and manage

**Consequences**:
- No user login/signup flow
- API keys must be managed externally (script generation)
- No fine-grained permissions (single-level access)
- Sufficient for MVP, may add OAuth in future

**Security Considerations**:
- Keys hashed with SHA-256 before storage
- Never log keys in plaintext
- Rate limiting per key prevents abuse

---

### ADR-005: Fly.io for Cloud Deployment

**Status**: Accepted
**Date**: 2025-11-25
**Context**: Need cloud deployment platform for Tiers 1-2

**Decision**: Use Fly.io for managed deployments

**Rationale**:
- Simple deployment (`fly deploy`)
- Automatic HTTPS with certificates
- Global edge network for low latency
- Rust-friendly with fast cold starts
- Free tier for small deployments

**Consequences**:
- Vendor lock-in (mitigated by Docker containerization)
- Less control than raw Kubernetes
- Pricing scales with usage
- Easy migration path: Fly.io → Kubernetes if needed

**Alternatives**:
- Tier 3: Self-hosted Kubernetes for full control
- Supabase: Database only, not application hosting

---

### ADR-006: 30-Day Location Retention

**Status**: Accepted
**Date**: 2025-11-25
**Context**: Need data retention policy for location privacy

**Decision**: Automatically delete locations older than 30 days

**Rationale**:
- Privacy-first approach (limits data accumulation)
- Sufficient for family safety use cases (recent locations most valuable)
- Reduces storage costs
- GDPR-friendly (automatic deletion)

**Consequences**:
- No long-term location history queries
- Users must export data before 30 days if needed
- Background job required for cleanup
- Configurable via `PM__LIMITS__LOCATION_RETENTION_DAYS`

**Implementation**:
- Hourly background job: `DELETE FROM locations WHERE created_at < NOW() - INTERVAL '30 days'`
- Batched deletes (10K rows at a time) to avoid long locks

---

### ADR-007: In-Memory Rate Limiting (Single-Instance)

**Status**: Accepted (with known limitation)
**Date**: 2025-11-25
**Context**: Need rate limiting to prevent API abuse

**Decision**: In-memory rate limiting using `governor` crate for MVP

**Rationale**:
- Simple implementation without external dependencies
- Sufficient for single-instance deployments (Tier 1)
- No Redis overhead

**Consequences**:
- **Limitation**: Doesn't work for multi-instance deployments (each instance has separate state)
- **Migration Path**: Add Redis-backed rate limiter for horizontal scaling (Tier 2-3)
- Good enough for MVP, documented as technical debt

**Future Enhancement**:
- Implement `RedisRateLimiter` for shared state across instances
- Fallback to in-memory if Redis unavailable

## Implementation Guidance

### Development Workflow

**Initial Setup**:
```bash
# 1. Clone repository
git clone https://github.com/hanibalsk/phone-manager-backend
cd phone-manager-backend

# 2. Install Rust toolchain
rustup default stable
rustup component add rustfmt clippy

# 3. Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# 4. Start PostgreSQL (via Docker)
docker-compose up -d db

# 5. Create database and run migrations
sqlx database create
sqlx migrate run --source crates/persistence/src/migrations

# 6. Generate SQLx offline query data (for CI)
cargo sqlx prepare --workspace

# 7. Build and run
cargo run --bin phone-manager
```

**Daily Development**:
```bash
# Run with auto-reload (requires cargo-watch)
cargo watch -x 'run --bin phone-manager'

# Run tests
cargo test --workspace

# Run specific test with output
cargo test test_device_registration -- --nocapture

# Lint and format
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

### Code Organization Patterns

**Route Handler Pattern**:
```rust
// crates/api/src/routes/devices.rs
pub async fn register_device(
    State(state): State<AppState>,
    auth: ApiKeyAuth,
    Json(payload): Json<DeviceRegistrationRequest>,
) -> Result<Json<DeviceRegistrationResponse>, ApiError> {
    // 1. Validate (automatic via serde + validator)
    payload.validate().map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // 2. Call service
    let service = DeviceService::new(&state.pool);
    let device = service.register(payload).await?;

    // 3. Map to response
    Ok(Json(device.into()))
}
```

**Service Pattern**:
```rust
// crates/domain/src/services/device.rs
pub struct DeviceService<'a> {
    repo: DeviceRepository<'a>,
}

impl<'a> DeviceService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            repo: DeviceRepository::new(pool),
        }
    }

    pub async fn register(&self, req: DeviceRegistrationRequest) -> Result<Device, DomainError> {
        // Business logic
        self.validate_group_size(&req.group_id).await?;
        self.repo.upsert_device(req).await
    }
}
```

**Repository Pattern**:
```rust
// crates/persistence/src/repositories/device.rs
pub struct DeviceRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> DeviceRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_device(&self, req: DeviceRegistrationRequest) -> Result<Device, DomainError> {
        // SQLx query with compile-time validation
        let entity = sqlx::query_as!(DeviceEntity, "...").fetch_one(self.pool).await?;
        Ok(entity.into())
    }
}
```

### Testing Patterns

**Unit Test** (domain logic):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_id_validation() {
        assert!(validate_group_id("family-123").is_ok());
        assert!(validate_group_id("invalid spaces").is_err());
    }
}
```

**Integration Test** (full API):
```rust
// tests/api_tests.rs
#[tokio::test]
async fn test_device_registration() {
    let server = common::setup_test_server().await;
    let api_key = common::create_test_api_key(&server).await;

    let response = server
        .post("/api/v1/devices/register")
        .add_header("X-API-Key", &api_key)
        .json(&json!({
            "device_id": "550e8400-e29b-41d4-a716-446655440000",
            "display_name": "Test Phone",
            "group_id": "test-group"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
}
```

### Error Handling Patterns

**Propagate Errors Up**:
```rust
// Repository returns Result
pub async fn get_device(&self, device_id: Uuid) -> Result<Device, DomainError> {
    let entity = sqlx::query_as!(DeviceEntity, "...")
        .fetch_optional(self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

    entity.ok_or(DomainError::DeviceNotFound(device_id))
}

// Service propagates
pub async fn get_device(&self, device_id: Uuid) -> Result<Device, DomainError> {
    self.repo.get_device(device_id).await
    // Add business logic here if needed
}

// Route converts to HTTP
pub async fn get_device(...) -> Result<Json<DeviceResponse>, ApiError> {
    let device = service.get_device(device_id).await?; // ? converts DomainError to ApiError
    Ok(Json(device.into()))
}
```

### Database Migration Workflow

**Creating New Migration**:
```bash
# Create migration file
sqlx migrate add create_new_table

# Edit crates/persistence/src/migrations/XXXXXX_create_new_table.sql
# Write SQL

# Test migration
sqlx migrate run --source crates/persistence/src/migrations

# Revert if needed
sqlx migrate revert --source crates/persistence/src/migrations

# Update offline query data
cargo sqlx prepare --workspace
```

**Migration Best Practices**:
- Always add indexes for foreign keys
- Include CHECK constraints for validation
- Use TIMESTAMPTZ for timestamps
- Add updated_at triggers where needed
- Test migrations on fresh database

### Performance Optimization Guidelines

**Database Query Optimization**:
1. Always use indexes for WHERE clauses
2. Use EXPLAIN ANALYZE to validate query plans
3. Prefer covering indexes for hot queries
4. Use partial indexes for filtered queries (e.g., active=true)
5. Batch operations where possible (location uploads)

**API Response Optimization**:
1. Use zero-copy serialization (serde's borrow feature)
2. Async I/O throughout (never block tokio runtime)
3. Connection pooling (reuse database connections)
4. Compression for large responses (gzip via tower-http)

**Memory Management**:
1. Use `&str` instead of `String` where possible
2. Avoid cloning large structures unnecessarily
3. Stream large responses (data export)
4. Limit request body size (1MB max)

### Deployment Checklist

**Pre-Deployment**:
- [ ] All tests pass (`cargo test --workspace`)
- [ ] No clippy warnings (`cargo clippy --workspace -- -D warnings`)
- [ ] Code formatted (`cargo fmt --all --check`)
- [ ] SQLx offline data updated (`cargo sqlx prepare --workspace`)
- [ ] Environment variables documented in `.env.example`
- [ ] Database migrations tested
- [ ] Docker image builds successfully
- [ ] Load tests pass (if available)

**Deployment Steps**:

**Tier 1 (Fly.io + Supabase)**:
```bash
# 1. Create Supabase project, get connection string
# 2. Configure Fly.io app
fly launch --name phone-manager-backend

# 3. Set secrets
fly secrets set PM__DATABASE__URL="postgres://..."

# 4. Deploy
fly deploy

# 5. Run migrations
fly ssh console
./phone-manager # migrations run on startup
```

**Tier 3 (Kubernetes)**:
```bash
# 1. Build and push Docker image
docker build -t phone-manager:latest .
docker push registry.example.com/phone-manager:latest

# 2. Apply Kubernetes manifests
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml

# 3. Verify deployment
kubectl get pods -n phone-manager
kubectl logs -f deployment/phone-manager -n phone-manager
```

### Monitoring Setup

**Prometheus Configuration**:
```yaml
scrape_configs:
  - job_name: 'phone-manager'
    static_configs:
      - targets: ['api.phonemanager.example.com']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

**Grafana Dashboards** (recommended metrics):
- API request rate (requests/second)
- API latency (p50, p95, p99)
- Error rate (%)
- Database connection pool usage
- Active devices per group
- Locations uploaded (per hour)
- Background job execution duration

## Proposed Source Tree

```
phone-manager-backend/
├── .github/
│   └── workflows/
│       ├── ci.yml                    # Build, test, lint on PR
│       ├── deploy-staging.yml        # Auto-deploy to staging
│       └── deploy-production.yml     # Manual deploy to prod
│
├── crates/
│   ├── api/                          # Binary crate (HTTP layer)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs               # Entry point, server startup
│   │       ├── lib.rs                # Library exports for testing
│   │       ├── app.rs                # Application builder (router + state)
│   │       ├── config.rs             # Configuration loading
│   │       ├── error.rs              # API error types + IntoResponse
│   │       │
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── devices.rs        # Device endpoints (register, list, delete)
│   │       │   ├── locations.rs      # Location endpoints (single, batch)
│   │       │   ├── admin.rs          # Admin endpoints (bulk ops, data export)
│   │       │   └── health.rs         # Health checks + metrics
│   │       │
│   │       ├── middleware/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs           # API key authentication
│   │       │   ├── logging.rs        # Request/response logging setup
│   │       │   ├── rate_limit.rs     # Per-key rate limiting
│   │       │   ├── trace_id.rs       # Request ID generation/extraction
│   │       │   └── audit.rs          # Audit logging for admin ops
│   │       │
│   │       ├── extractors/
│   │       │   ├── mod.rs
│   │       │   ├── api_key.rs        # ApiKeyAuth extractor
│   │       │   └── validated_json.rs # JSON + validation extractor
│   │       │
│   │       └── jobs/
│   │           ├── mod.rs
│   │           ├── scheduler.rs      # Job scheduler with tokio intervals
│   │           ├── location_cleanup.rs  # Delete old locations
│   │           └── matview_refresh.rs   # Refresh materialized views
│   │
│   ├── domain/                       # Library crate (business logic)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── errors.rs             # Domain error types
│   │       │
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── device.rs         # Device domain model + DTOs
│   │       │   ├── location.rs       # Location domain model + DTOs
│   │       │   ├── api_key.rs        # API key domain model
│   │       │   └── responses.rs      # Shared response types
│   │       │
│   │       └── services/
│   │           ├── mod.rs
│   │           ├── device.rs         # Device service (registration, validation)
│   │           ├── location.rs       # Location service (upload, batch)
│   │           └── group.rs          # Group service (listing, validation)
│   │
│   ├── persistence/                  # Library crate (data access)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── db.rs                 # Connection pool creation
│   │       ├── errors.rs             # Persistence error types
│   │       │
│   │       ├── entities/
│   │       │   ├── mod.rs
│   │       │   ├── device.rs         # DeviceEntity (FromRow)
│   │       │   ├── location.rs       # LocationEntity (FromRow)
│   │       │   └── api_key.rs        # ApiKeyEntity (FromRow)
│   │       │
│   │       ├── repositories/
│   │       │   ├── mod.rs
│   │       │   ├── device.rs         # Device CRUD operations
│   │       │   ├── location.rs       # Location inserts, queries
│   │       │   └── api_key.rs        # API key lookup, validation
│   │       │
│   │       └── migrations/           # SQLx migrations
│   │           ├── 001_initial.sql
│   │           ├── 002_devices.sql
│   │           ├── 003_locations.sql
│   │           ├── 004_api_keys.sql
│   │           └── 005_views_and_functions.sql
│   │
│   └── shared/                       # Library crate (utilities)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── crypto.rs             # API key hashing (SHA-256)
│           ├── time.rs               # Time utilities, conversions
│           └── validation.rs         # Custom validators
│
├── tests/                            # Integration tests
│   ├── common/
│   │   └── mod.rs                    # Test utilities (setup, fixtures)
│   ├── api_tests.rs                  # API endpoint tests
│   ├── device_tests.rs               # Device-specific scenarios
│   ├── location_tests.rs             # Location upload scenarios
│   └── fixtures/
│       └── test_data.rs              # Test data generators
│
├── config/                           # Configuration files
│   ├── default.toml                  # Default configuration
│   └── minimal.toml                  # Minimal deployment config
│
├── k8s/                              # Kubernetes manifests (Tier 3)
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secret.yaml.example           # Template (never commit actual secrets)
│   ├── deployment.yaml               # Deployment with 3 replicas
│   ├── service.yaml                  # ClusterIP service
│   ├── ingress.yaml                  # Ingress with TLS
│   └── hpa.yaml                      # Horizontal Pod Autoscaler
│
├── scripts/                          # Utility scripts
│   ├── setup-db.sh                   # Database setup automation
│   ├── generate-api-key.sh           # API key generation
│   └── load-test.sh                  # k6 load testing script
│
├── docs/                             # Documentation
│   ├── PRD.md                        # Product Requirements Document
│   ├── epics.md                      # Epic breakdown with stories
│   ├── solution-architecture.md     # This document
│   ├── rust-backend-spec.md         # Original technical spec
│   ├── project-workflow-analysis.md # Project assessment
│   └── api-examples.md              # API usage examples (future)
│
├── Cargo.toml                        # Workspace configuration
├── Cargo.lock                        # Dependency lock (committed for binaries)
├── rust-toolchain.toml               # Rust version pinning
├── .env.example                      # Environment template
├── .gitignore                        # Git ignore rules
├── Dockerfile                        # Multi-stage Docker build
├── docker-compose.yml                # Local development environment
├── fly.toml                          # Fly.io deployment config
├── sqlx-data.json                    # SQLx offline query metadata (generated)
├── CLAUDE.md                         # Claude Code context
└── README.md                         # Project documentation
```

### File Count Estimates

| Directory | Estimated Files | Purpose |
|-----------|----------------|---------|
| `crates/api/src/` | ~15 files | HTTP layer, middleware, routes |
| `crates/domain/src/` | ~10 files | Business logic, models, services |
| `crates/persistence/src/` | ~12 files | Repositories, entities, migrations |
| `crates/shared/src/` | ~4 files | Common utilities |
| `tests/` | ~8 files | Integration tests |
| `k8s/` | ~7 files | Kubernetes manifests |
| `docs/` | ~6 files | Documentation |
| **Total** | **~60-70 files** | Complete implementation |

### Key Implementation Files

| File | Purpose | Epic |
|------|---------|------|
| `crates/api/src/main.rs` | Application entry point | Epic 1 |
| `crates/api/src/app.rs` | Router + middleware setup | Epic 1 |
| `crates/api/src/config.rs` | Configuration management | Epic 1 |
| `crates/api/src/middleware/auth.rs` | API key authentication | Epic 1 |
| `crates/api/src/routes/devices.rs` | Device endpoints | Epic 2 |
| `crates/api/src/routes/locations.rs` | Location endpoints | Epic 3 |
| `crates/domain/src/services/device.rs` | Device business logic | Epic 2 |
| `crates/domain/src/services/location.rs` | Location business logic | Epic 3 |
| `crates/persistence/src/repositories/device.rs` | Device data access | Epic 2 |
| `crates/persistence/src/repositories/location.rs` | Location data access | Epic 3 |
| `crates/api/src/jobs/scheduler.rs` | Background job system | Epic 3 |
| `crates/persistence/src/migrations/*.sql` | Database schema | Epic 1 |

## DevOps and Deployment

### CI/CD Pipeline (GitHub Actions)

**Workflow: `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: phone_manager_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Run tests
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/phone_manager_test
        run: cargo test --workspace

      - name: Build
        run: cargo build --release --bin phone-manager

      - name: Validate SQLx offline data
        run: cargo sqlx prepare --check --workspace
```

**Workflow: `.github/workflows/deploy-staging.yml`**

```yaml
name: Deploy to Staging

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Deploy to Fly.io
        uses: superfly/flyctl-actions/setup-flyctl@master
      - run: flyctl deploy --remote-only --app phone-manager-staging
        env:
          FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}
```

### Docker Configuration

**Dockerfile** (multi-stage build):

```dockerfile
# Build stage
FROM rust:1.83-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

# Build dependencies (cached layer)
RUN cargo build --release --bin phone-manager

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/phone-manager /app/phone-manager

# Copy configuration
COPY config ./config

# Create non-root user
RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health/live || exit 1

CMD ["./phone-manager"]
```

**docker-compose.yml** (development):

```yaml
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
    volumes:
      - ./config:/app/config:ro

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
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

### Kubernetes Configuration

**Deployment** (`k8s/deployment.yaml`):

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: phone-manager
  namespace: phone-manager
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: phone-manager
  template:
    metadata:
      labels:
        app: phone-manager
    spec:
      containers:
      - name: api
        image: registry.example.com/phone-manager:latest
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
        - name: PM__LOGGING__FORMAT
          value: "json"
        resources:
          requests:
            cpu: 250m
            memory: 256Mi
          limits:
            cpu: 500m
            memory: 512Mi
        livenessProbe:
          httpGet:
            path: /api/health/live
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /api/health/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

**Horizontal Pod Autoscaler** (`k8s/hpa.yaml`):

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: phone-manager-hpa
  namespace: phone-manager
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: phone-manager
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### Fly.io Configuration

**fly.toml**:

```toml
app = "phone-manager-backend"
primary_region = "ams" # Amsterdam (closest to Europe)

[build]
  dockerfile = "Dockerfile"

[env]
  PM__SERVER__PORT = "8080"
  PM__LOGGING__LEVEL = "info"
  PM__LOGGING__FORMAT = "json"

[[services]]
  http_checks = []
  internal_port = 8080
  protocol = "tcp"
  script_checks = []

  [services.concurrency]
    hard_limit = 250
    soft_limit = 200
    type = "connections"

  [[services.ports]]
    force_https = true
    handlers = ["http"]
    port = 80

  [[services.ports]]
    handlers = ["tls", "http"]
    port = 443

  [[services.tcp_checks]]
    grace_period = "10s"
    interval = "30s"
    restart_limit = 0
    timeout = "5s"

  [[services.http_checks]]
    interval = "30s"
    grace_period = "10s"
    method = "get"
    path = "/api/health/ready"
    protocol = "http"
    timeout = "5s"
```

### Environment Configuration

**Development** (`.env`):
```bash
PM__SERVER__HOST=0.0.0.0
PM__SERVER__PORT=8080
PM__DATABASE__URL=postgres://postgres:postgres@localhost:5432/phone_manager
PM__LOGGING__LEVEL=debug
PM__LOGGING__FORMAT=pretty
PM__SECURITY__CORS_ORIGINS=*
```

**Production** (environment variables):
```bash
PM__SERVER__HOST=0.0.0.0
PM__SERVER__PORT=8080
PM__DATABASE__URL=<secret>
PM__LOGGING__LEVEL=info
PM__LOGGING__FORMAT=json
PM__SECURITY__CORS_ORIGINS=https://app.phonemanager.com
PM__SECURITY__RATE_LIMIT_PER_MINUTE=100
PM__LIMITS__MAX_DEVICES_PER_GROUP=20
PM__LIMITS__MAX_BATCH_SIZE=50
PM__LIMITS__LOCATION_RETENTION_DAYS=30
```

### Monitoring and Alerting

**Prometheus Alerts** (recommended):

```yaml
groups:
  - name: phone-manager
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        annotations:
          summary: "High error rate detected"

      - alert: HighLatency
        expr: histogram_quantile(0.95, http_request_duration_seconds) > 0.2
        for: 5m
        annotations:
          summary: "API latency above 200ms p95"

      - alert: DatabaseDown
        expr: up{job="phone-manager"} == 0
        for: 1m
        annotations:
          summary: "Phone Manager API is down"
```

**Grafana Dashboard Panels**:
1. Request rate (requests/sec)
2. Latency percentiles (p50, p95, p99)
3. Error rate (%)
4. Active database connections
5. Locations uploaded (per hour)
6. Active devices count
7. Background job execution time

### Backup and Disaster Recovery

**Database Backup Strategy**:

**Automated Backups**:
- **Frequency**: Daily full backup at 2 AM UTC
- **Method**: `pg_dump` to S3-compatible storage
- **Retention**: 7 days for daily, 4 weeks for weekly
- **Encryption**: AES-256 encryption at rest

**Point-in-Time Recovery**:
- **WAL Archiving**: Continuous archiving to S3
- **Recovery Window**: 7 days
- **RTO**: 1 hour (recovery time objective)
- **RPO**: 5 minutes (recovery point objective)

**Backup Script** (`scripts/backup-db.sh`):
```bash
#!/bin/bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="phone_manager_${TIMESTAMP}.sql.gz"

pg_dump $PM__DATABASE__URL | gzip > $BACKUP_FILE
aws s3 cp $BACKUP_FILE s3://backups/phone-manager/$BACKUP_FILE
```

### Secrets Management

**Development**:
- `.env` file (git-ignored)
- Never commit secrets to repository

**Fly.io**:
```bash
fly secrets set PM__DATABASE__URL="postgres://..."
fly secrets set PM__API_KEY_SECRET="..."
```

**Kubernetes**:
```bash
kubectl create secret generic phone-manager-secrets \
  --from-literal=database-url="postgres://..." \
  --namespace phone-manager
```

**Best Practices**:
- Rotate database credentials every 90 days
- Use separate credentials per environment
- Never log secrets
- Use secret scanning in CI (e.g., GitGuardian)

## Security Architecture

### Threat Model

**Assets to Protect**:
1. Location data (sensitive personal information)
2. Device identities and group memberships
3. API keys
4. Database credentials

**Threat Actors**:
1. Unauthorized API access (external attackers)
2. Rate limit abuse / DoS attempts
3. SQL injection attempts
4. Data exfiltration

**Attack Vectors**:
1. Missing/stolen API keys
2. Malicious payloads (SQL injection, XSS)
3. Rate limit exhaustion
4. Man-in-the-middle attacks

### Security Controls

#### Layer 1: Network Security
- **TLS 1.3**: All communication encrypted (terminated at load balancer)
- **HTTPS Only**: HTTP redirects to HTTPS in production
- **Firewall**: Database not publicly accessible (VPC/private network only)

#### Layer 2: Authentication & Authorization
- **API Key Required**: All endpoints except health checks
- **SHA-256 Hashing**: Keys never stored in plaintext
- **Key Expiration**: Optional expiration dates enforced
- **Admin Keys**: Separate `is_admin` flag for privileged operations

#### Layer 3: Input Validation
- **Type Safety**: Rust type system prevents type confusion
- **Validator Crate**: Declarative validation on all inputs
- **Database Constraints**: CHECK constraints as defense-in-depth
- **Parameterized Queries**: SQLx enforces prepared statements (no SQL injection)

#### Layer 4: Rate Limiting
- **Per-API Key**: Prevents single client abuse
- **Sliding Window**: More accurate than fixed window
- **Configurable Limits**: Adjust per deployment tier
- **Graceful 429**: Clear Retry-After guidance

#### Layer 5: Data Protection
- **Encryption at Rest**: Database volume encryption
- **Encryption in Transit**: TLS for all connections
- **Automatic Deletion**: 30-day retention limit
- **Audit Logging**: Track all privacy-sensitive operations

### Security Headers

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'none'; frame-ancestors 'none'
```

### GDPR Compliance (FR-24)

**Right to Access**:
- Endpoint: `GET /api/v1/devices/:device_id/data-export`
- Returns: Complete data export in machine-readable format (JSON)
- Timeline: Immediate (real-time query)

**Right to Erasure**:
- Endpoint: `DELETE /api/v1/devices/:device_id/data`
- Action: Hard delete device + cascade delete all locations
- Timeline: Immediate
- Audit: Logged with API key ID, timestamp, IP address

**Right to Be Informed**:
- Privacy policy (future): Document what data is collected, retention policy
- API documentation: Clear data handling practices
- Automatic deletion: Users informed of 30-day retention

**Data Minimization**:
- Only collect necessary location metadata
- No tracking of non-location user activity
- No third-party data sharing

### Vulnerability Management

**Dependency Scanning**:
- `cargo audit` in CI pipeline
- Automated security advisories from RustSec
- Update dependencies regularly

**Secret Scanning**:
- GitGuardian or GitHub secret scanning
- Pre-commit hooks to prevent secret commits
- `.env` in `.gitignore`

**Security Testing**:
- Input fuzzing (future)
- Penetration testing (post-MVP)
- Regular security reviews

## Testing Strategy

### Testing Pyramid

```
         ┌────────────────┐
        ╱    E2E Tests     ╲      10% - Full API scenarios
       ╱      (~10)         ╲
      ├────────────────────┤
     ╱  Integration Tests   ╲     30% - API endpoint tests
    ╱        (~30)           ╲
   ├──────────────────────────┤
  ╱      Unit Tests           ╲   60% - Domain logic, validators
 ╱         (~60)               ╲
└──────────────────────────────┘
```

**Target Coverage**: 80% overall, 90% for domain/business logic

### Unit Tests

**Scope**: Domain logic, validation, utilities (no I/O)

**Location**: `#[cfg(test)]` modules within each file

**Example**:
```rust
// crates/domain/src/models/device.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_id_validation() {
        assert!(validate_group_id("family-123").is_ok());
        assert!(validate_group_id("my_group").is_ok());
        assert!(validate_group_id("a").is_err()); // Too short
        assert!(validate_group_id("invalid spaces").is_err());
    }

    #[test]
    fn test_coordinate_validation() {
        let payload = LocationPayload {
            latitude: 91.0, // Invalid
            ..Default::default()
        };
        assert!(payload.validate().is_err());
    }
}
```

**What to Test**:
- Validation logic (group ID format, coordinate ranges)
- Business rules (group size limits)
- Domain model transformations
- Utility functions (hashing, time conversions)
- Error type conversions

### Integration Tests

**Scope**: Full API endpoints with test database

**Location**: `tests/` directory

**Setup**:
```rust
// tests/common/mod.rs
pub async fn setup_test_server() -> TestServer {
    let test_db = create_test_database().await;
    run_migrations(&test_db).await;

    let config = Config::test_config();
    let app = create_app(config, test_db);

    TestServer::new(app).unwrap()
}
```

**Example**:
```rust
#[tokio::test]
async fn test_full_device_registration_flow() {
    let server = setup_test_server().await;
    let api_key = create_test_api_key(&server).await;

    // Register device
    let response = server.post("/api/v1/devices/register")
        .add_header("X-API-Key", &api_key)
        .json(&device_payload())
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // Verify device appears in group listing
    let response = server.get("/api/v1/devices?group_id=test-group")
        .add_header("X-API-Key", &api_key)
        .await;

    let body: DevicesResponse = response.json();
    assert_eq!(body.devices.len(), 1);
}
```

**Test Scenarios**:
- Device registration → group listing → location upload → query
- Batch location upload with validation errors
- Rate limiting behavior (exceed limit, verify 429)
- Group size enforcement (21st device rejected)
- Unauthorized access (missing API key)
- Idempotency (duplicate requests return same response)

### End-to-End Tests

**Scope**: Complete user journeys from PRD

**Tools**: Integration test framework + test client

**Scenarios**:
1. **Parent Tracking Teen** (Journey 1):
   - Register 2 devices in same group
   - Upload location from device 1
   - Query group devices
   - Verify last_location present

2. **Road Trip Coordination** (Journey 2):
   - Register 9 devices in group
   - Attempt 10th device (should fail at 20 limit)
   - Batch upload 50 locations
   - Verify atomic transaction

3. **Elderly Care** (Journey 3):
   - Register monitoring devices
   - Upload locations over time
   - Export data (GDPR compliance)
   - Delete device and verify cascade

### Performance Testing

**Tool**: k6 load testing

**Scenarios**:

**Baseline Load**:
```javascript
// scripts/load-test.js
export let options = {
  vus: 100, // 100 virtual users
  duration: '5m',
  thresholds: {
    http_req_duration: ['p(95)<200'], // 95% under 200ms
    http_req_failed: ['rate<0.01'],   // <1% errors
  },
};

export default function() {
  // Mix of operations:
  // 40% location uploads
  // 30% group queries
  // 20% device registration
  // 10% batch uploads
}
```

**Stress Test**:
- Ramp up to 10,000 concurrent connections
- Sustain 1,000 requests/second
- Verify p95 latency <200ms
- Verify database connections <100

**Spike Test**:
- Sudden traffic spike (0 → 5,000 users in 10 seconds)
- Verify graceful handling
- Verify no errors during scale-up

### Test Database Management

**Strategy**: Isolated database per test

```rust
pub async fn create_test_database() -> PgPool {
    let test_db_name = format!("test_{}", Uuid::new_v4().simple());

    // Connect to postgres to create test DB
    let admin_pool = PgPool::connect("postgres://postgres@localhost/postgres").await.unwrap();
    sqlx::query(&format!("CREATE DATABASE {}", test_db_name))
        .execute(&admin_pool)
        .await
        .unwrap();

    // Connect to test DB
    let test_pool = PgPool::connect(&format!("postgres://postgres@localhost/{}", test_db_name))
        .await
        .unwrap();

    // Run migrations
    sqlx::migrate!("../persistence/src/migrations")
        .run(&test_pool)
        .await
        .unwrap();

    test_pool
}
```

**Cleanup**: Drop database after test completes (in Drop impl or test teardown)

### Test Fixtures

**Using `fake` crate**:
```rust
use fake::{Fake, Faker};
use uuid::Uuid;

pub fn fake_device_payload() -> DeviceRegistrationRequest {
    DeviceRegistrationRequest {
        device_id: Uuid::new_v4(),
        display_name: Faker.fake(),
        group_id: format!("group-{}", Faker.fake::<String>()),
        platform: "android".to_string(),
        fcm_token: Some(Faker.fake()),
    }
}

pub fn fake_location_payload(device_id: Uuid) -> LocationPayload {
    LocationPayload {
        device_id,
        timestamp: chrono::Utc::now().timestamp_millis(),
        latitude: (-90.0..90.0).fake(),
        longitude: (-180.0..180.0).fake(),
        accuracy: (1.0..100.0).fake(),
        ..Default::default()
    }
}
```

### Continuous Integration

**On Every PR**:
- [ ] Code formatting check (`cargo fmt --check`)
- [ ] Linting (`cargo clippy -- -D warnings`)
- [ ] Unit tests (`cargo test --lib`)
- [ ] Integration tests (`cargo test --test`)
- [ ] SQLx offline data validation
- [ ] Docker build succeeds

**On Main Branch Push**:
- [ ] All CI checks pass
- [ ] Auto-deploy to staging (Fly.io)
- [ ] Smoke tests on staging
- [ ] Optional: Load test on staging

**Before Production Deploy**:
- [ ] Manual approval required
- [ ] All tests pass on staging
- [ ] Performance benchmarks meet targets
- [ ] Security scan clean
- [ ] Database migration plan reviewed

---

_This architecture document provides the technical foundation for implementing the 32 stories defined in epics.md._
