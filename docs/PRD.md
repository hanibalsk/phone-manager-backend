# phone-manager-backend Product Requirements Document (PRD)

**Author:** Martin Janci
**Date:** 2025-11-25
**Project Level:** Level 3 (Full Product)
**Project Type:** Backend service/API (Rust Axum)
**Target Scale:** 20-30 stories, 4 epics, 6-week timeline

---

## Description, Context and Goals

**Phone Manager** is a backend API service that powers a family location-sharing mobile application. The system enables trusted groups of users (family members, close friends) to share their real-time locations with each other through a simple, privacy-conscious platform.

The core value proposition is **peace of mind** — parents can see where their children are, family members can coordinate meetups, and loved ones can ensure each other's safety during travel or emergencies.

**Key Capabilities:**
- **Device Registration** — Android devices register with a unique identity and join location-sharing groups
- **Real-time Location Tracking** — Devices upload GPS coordinates (single or batch) with metadata (accuracy, battery, network)
- **Group-based Sharing** — Devices organized into groups (max 20 members) for controlled visibility
- **Location History** — 30-day retention for reviewing past locations and patterns

**Target Users:**
- Families with children/teens
- Elderly care situations
- Close friend groups for coordination
- Small team coordination (field workers, delivery)

### Deployment Intent

Production SaaS/application with flexible deployment tiers:
- **Tier 1 (Minimal)**: Supabase + single container for small family deployments (<20 devices)
- **Tier 2 (Standard)**: Self-hosted PostgreSQL + Kubernetes for medium deployments (20-100 devices)
- **Tier 3 (Production)**: Full production stack with observability, monitoring, and scaling (100+ devices)

### Context

**The Problem**: Families and close-knit groups need a simple way to stay coordinated and ensure each other's safety through location sharing, but existing solutions (Life360, Google Family Link, Find My) are either proprietary, privacy-invasive, or tied to specific ecosystems. Self-hosting options are limited, and small groups don't want to commit to expensive commercial platforms with surveillance-heavy business models.

**Current Situation**: The Android mobile client is ready for backend integration. We need a robust, privacy-conscious API that handles device registration, location tracking, and group management while remaining simple enough for small-scale self-hosting yet scalable for production deployments.

**Why Now**: The demand for privacy-respecting alternatives to big-tech location services is growing. The Rust ecosystem has matured with production-ready frameworks (Axum, SQLx, Tokio), making it feasible to build a performant, type-safe backend that can be deployed anywhere from a home server (Supabase) to Kubernetes clusters. The mobile client is ready, creating urgency for a reliable backend.

### Goals

1. **Reliability**: Achieve 99.9% uptime with <200ms p95 API response time
2. **Scalability**: Support 10,000+ concurrent connections with efficient location batching
3. **Privacy-First**: Implement secure API key authentication with 30-day automatic data retention
4. **Developer Experience**: Provide comprehensive API documentation and deployment flexibility (Supabase to Kubernetes)
5. **Production Readiness**: Ship with observability (metrics, logging, tracing) built-in from day one

## Requirements

### Functional Requirements

#### Device Management
1. **FR-1: Device Registration** — Mobile clients can register devices with unique UUID, display name (2-50 chars), group ID (alphanumeric, hyphens, underscores), platform identifier, and optional FCM token
2. **FR-2: Device Updates** — Devices can update their display name and FCM token via re-registration
3. **FR-3: Group Membership** — Devices automatically join groups upon registration with validation of group size limits (max 20 devices per group)
4. **FR-4: Device Deactivation** — API supports soft delete with active=false flag; locations remain queryable but device excluded from active group listings

#### Location Tracking
5. **FR-5: Single Location Upload** — Devices can POST location data including latitude (-90 to 90), longitude (-180 to 180), accuracy (meters), optional altitude, bearing (0-360°), speed (m/s), timestamp, battery level (0-100%), network type, and provider
6. **FR-6: Batch Location Upload** — Devices can upload up to 50 locations in a single request (max 1MB payload) for offline-to-online sync scenarios with 30-second timeout
7. **FR-7: Location Validation** — System validates coordinate ranges, accuracy values, and timestamps before storage
8. **FR-8: Last Location Retrieval** — API returns each device's most recent location when querying group members

#### Group Operations
9. **FR-9: Group Device Listing** — Retrieve all active devices in a group with their last known location, last seen timestamp, and display names
10. **FR-10: Group Size Enforcement** — Prevent registration when group has reached 20 device limit with clear error message
11. **FR-11: Last Activity Tracking** — Track last_seen_at timestamp updated on any authenticated API call from device

#### Data Lifecycle
12. **FR-12: Location Retention** — Automatically delete location records older than 30 days via background job

#### Authentication & Security
14. **FR-14: API Key Authentication** — All endpoints (except health checks) require valid API key in X-API-Key header
15. **FR-15: API Key Management** — Support creation, activation/deactivation, expiration, and rotation of API keys
16. **FR-16: Rate Limiting** — Enforce rate limits per API key across all endpoints (default: 100 requests/minute); return 429 with Retry-After header when exceeded

#### Observability
17. **FR-17: Health Endpoints** — Provide /health, /health/live, and /health/ready endpoints for monitoring and orchestration
18. **FR-18: Metrics Export** — Expose Prometheus-compatible metrics for API response times, request counts, error rates, and database health

#### System Infrastructure
19. **FR-19: Transaction Support** — Batch location uploads (FR-6) must be atomic (all succeed or all fail) to prevent partial data corruption
20. **FR-20: Background Job Scheduler** — System must support scheduled tasks for location retention cleanup (FR-12) and materialized view refreshes
21. **FR-21: Error Response Format** — All API errors return consistent JSON structure with error code, message, and optional details array for validation errors
22. **FR-22: Request Idempotency** — Location uploads (FR-5, FR-6) support idempotency keys to prevent duplicate location records from retries
23. **FR-23: Device Migration** — Devices can change groups with validation; old group membership terminated and location history preserved
24. **FR-24: Data Privacy Controls** — Support device data export (all locations) and complete deletion (device + all locations) for privacy compliance
25. **FR-25: API Versioning** — API endpoints include version prefix (/api/v1/) to support backward compatibility during evolution
26. **FR-26: Admin Operations** — Support bulk device cleanup (delete inactive devices older than configurable threshold) and group management

### Non-Functional Requirements

#### Performance
1. **NFR-1: API Response Time** — 95th percentile (p95) API response time must be <200ms for all endpoints under normal load
2. **NFR-2: Throughput** — System must handle 10,000+ concurrent connections with sustained throughput of 1,000 requests/second
3. **NFR-3: Database Query Performance** — All database queries must complete within 100ms at p95 for optimal API response times
4. **NFR-4: Batch Processing Efficiency** — Batch location uploads (up to 50 locations) must complete within 500ms at p95

#### Reliability & Availability
5. **NFR-5: Uptime** — System must achieve 99.9% uptime (max 8.7 hours downtime per year) for Tier 2-3 deployments
6. **NFR-6: Data Durability** — All location data must be persisted with ACID guarantees; zero data loss on system failure
7. **NFR-7: Graceful Degradation** — System must continue serving read requests when write operations fail

#### Scalability
8. **NFR-8: Horizontal Scaling** — Architecture must support horizontal scaling by adding API server instances without code changes
9. **NFR-9: Database Scaling** — Support PostgreSQL connection pooling (20-100 connections) with efficient query patterns for read replicas
10. **NFR-10: Storage Growth** — System must efficiently handle 1M+ location records per month with automated cleanup maintaining performance

#### Security
11. **NFR-11: Authentication** — All API endpoints (except health checks) must enforce API key authentication with SHA-256 hashed storage
12. **NFR-12: Data Privacy** — Support GDPR-compliant data export and deletion within 30 days of request
13. **NFR-13: Secure Communication** — All client-server communication must use TLS 1.3+ in production deployments
14. **NFR-14: Rate Limiting** — Enforce configurable rate limits per API key to prevent abuse (default: 100 req/min)

#### Operational Excellence
15. **NFR-15: Observability** — Expose Prometheus metrics, structured JSON logs, and distributed tracing for all requests
16. **NFR-16: Deployment Flexibility** — Support multiple deployment tiers (Supabase minimal, self-hosted standard, production Kubernetes)
17. **NFR-17: Zero-Downtime Deployments** — Support rolling updates without service interruption for Tier 2-3 deployments

## User Journeys

#### Journey 1: Parent Tracking Teen's Location (Primary Use Case)

**Persona**: Sarah, 42-year-old parent of 16-year-old daughter Emma

**Goal**: Know when Emma arrives home safely from school

**Journey Flow**:

1. **Setup Phase**
   - Sarah installs Phone Manager app on her phone and Emma's phone
   - Both devices register with backend (FR-1) joining group "family-sarah"
   - Backend validates group creation and returns success

2. **Daily Monitoring**
   - Sarah opens app to check Emma's location
   - App queries backend for group devices (FR-9)
   - Backend returns Emma's last location with timestamp and battery level (FR-8, FR-11)
   - Sarah sees Emma is at school, battery at 65%, last updated 5 minutes ago

3. **Location Updates (Emma's Device)**
   - Emma's phone sends location every 5 minutes in background
   - Batch upload accumulates offline locations (FR-6)
   - When online, sends batch of 12 locations from past hour
   - Backend validates and stores atomically (FR-19, FR-7)

4. **Arrival Notification Scenario**
   - Emma arrives home at 3:45 PM
   - Emma's device uploads location showing home coordinates
   - Sarah refreshes app and sees Emma at home
   - **Decision Point**: Sarah feels reassured and stops monitoring

5. **Privacy Respect**
   - After 30 days, old location records auto-delete (FR-12)
   - Emma can request data export or deletion via app (FR-24)

**Pain Points Addressed**:
- No expensive subscription (self-hosted option)
- Emma's historical data doesn't accumulate indefinitely
- Simple setup without complex configuration

---

#### Journey 2: Family Group Coordination During Road Trip

**Persona**: Miguel, 38-year-old organizing family reunion road trip with 3 vehicles

**Goal**: Coordinate multiple vehicles traveling to reunion location

**Journey Flow**:

1. **Group Formation**
   - Miguel creates group "reunion-2025"
   - Shares group ID with 8 family members across 3 vehicles
   - All devices register to same group (FR-3)
   - Backend enforces 20-device limit (FR-10)

2. **En Route Tracking**
   - Lead car (Miguel) checks group map every 30 minutes
   - Backend returns all 9 devices with last locations (FR-9)
   - Miguel sees one car falling behind
   - **Decision Point**: Miguel slows down or suggests stop

3. **Poor Connectivity Scenario**
   - One vehicle enters area with spotty coverage
   - Device queues locations locally
   - When connectivity returns, batch uploads 45 locations (FR-6)
   - Backend processes atomically with idempotency (FR-22, FR-19)

4. **Arrival & Cleanup**
   - All vehicles arrive safely
   - Group continues monitoring for duration of reunion (1 week)
   - After reunion, devices leave group or go inactive (FR-4)
   - 30 days later, trip location data auto-deletes (FR-12)

**Pain Points Addressed**:
- Works with intermittent connectivity
- No manual location sharing via text/calls
- Automatic cleanup prevents data accumulation

---

#### Journey 3: Elderly Care Monitoring (Secondary Use Case)

**Persona**: Lisa, 50-year-old daughter monitoring 78-year-old father with early dementia

**Goal**: Ensure father doesn't wander beyond safe area without intrusive monitoring

**Journey Flow**:

1. **Setup for Monitoring**
   - Lisa sets up father's phone with Phone Manager
   - Both join group "lisa-dad-care"
   - Father doesn't need to interact with app (runs in background)

2. **Periodic Check-ins**
   - Lisa checks location 3-4 times per day
   - Backend returns father's last location (FR-8)
   - Timestamp shows when last updated (FR-11)
   - **Decision Point**: If no update >2 hours, Lisa calls to check

3. **Anomaly Detection** (Future: Out of Scope for MVP)
   - Father's location shows movement to unfamiliar area
   - Lisa receives notification (FR not yet implemented)
   - Lisa calls father to check if he's okay

4. **Privacy & Trust**
   - Father aware of monitoring (ethical requirement)
   - Lisa can export location history if needed for medical records (FR-24)
   - Data auto-deletes after 30 days (FR-12)

**Pain Points Addressed**:
- Peace of mind without invasive tracking
- Works passively without father needing tech skills
- Data export for medical/legal purposes if needed

## UX Design Principles

1. **Privacy by Design** — Location data is sensitive personal information. Default to minimal data collection, automatic deletion after 30 days, and transparent data handling. Users must always know what data is collected and have control over deletion.

2. **Offline-First Resilience** — Mobile devices experience intermittent connectivity. The system must gracefully handle offline scenarios with batch upload support, idempotency guarantees, and clear feedback on sync status.

3. **Minimal Cognitive Load** — Location sharing should "just work" in the background. Avoid requiring users to understand technical concepts like API keys, group IDs (use friendly names in UI), or coordinate systems.

4. **Performance as a Feature** — Fast response times (<200ms) translate to responsive UI. Every millisecond of API latency directly impacts user experience, especially during critical "where are they?" moments.

5. **Graceful Error Communication** — When things fail (rate limit, invalid data, network errors), provide clear, actionable error messages through structured error responses (FR-21). Never expose technical jargon to end users.

6. **Trust Through Transparency** — Location tracking can feel invasive. Build trust by showing last update timestamps, battery levels, and location accuracy. Make data retention policies visible and enforceable.

7. **Accessibility and Inclusivity** — Support users with varying technical literacy (from teens to elderly). Design API responses to enable simple UI patterns that work for all age groups and abilities.

8. **Scalable Simplicity** — Start simple (Tier 1: family of 4) and scale complexity gradually (Tier 3: extended family of 20). Architecture must support both without forcing complexity on small deployments.

9. **Respectful Rate Limiting** — Rate limits protect the system but shouldn't punish legitimate use. Provide clear Retry-After headers and generous limits (100 req/min) that accommodate normal family usage patterns.

10. **Developer Experience Matters** — A well-documented, predictable API enables better client applications. Consistent response formats, semantic HTTP codes, and versioning (FR-25) empower developers to build great experiences.

## Epics

#### Epic 1: Foundation & Core API Infrastructure
**Goal**: Establish production-ready API infrastructure with authentication, configuration, and health monitoring

**Business Value**: Enables all subsequent development; establishes security and observability baseline

**Story Count**: ~8 stories

**Key Capabilities**:
- Project workspace structure with Rust 2024, Axum, SQLx, Tokio
- Configuration management (TOML + environment variables)
- PostgreSQL database setup with migrations
- API key authentication middleware
- Health check endpoints (/health, /health/live, /health/ready)
- Structured logging with tracing
- Error handling framework (FR-21)
- Docker development environment

**Acceptance Criteria**:
- All endpoints return 401 without valid API key
- Health checks return 200 with database connectivity status
- Configuration loads from files and environment
- Logs structured JSON in production mode

---

#### Epic 2: Device Management
**Goal**: Enable mobile devices to register, update, and manage group membership

**Business Value**: Core prerequisite for location tracking; establishes user identity

**Story Count**: ~6 stories

**Key Capabilities**:
- Device registration API (FR-1)
- Device update via re-registration (FR-2)
- Group membership validation (FR-3)
- Group size enforcement with 20-device limit (FR-10)
- Device soft delete/deactivation (FR-4)
- Device migration between groups (FR-23)

**Acceptance Criteria**:
- Devices can register with UUID, name, group ID, FCM token
- Registration fails when group at 20-device capacity
- Device updates preserve location history
- Inactive devices excluded from active group listings

---

#### Epic 3: Location Tracking & Retrieval
**Goal**: Enable devices to upload locations and users to query group member locations

**Business Value**: Core product functionality; delivers on "peace of mind" value proposition

**Story Count**: ~10 stories

**Key Capabilities**:
- Single location upload API (FR-5)
- Batch location upload with atomic transactions (FR-6, FR-19)
- Coordinate and metadata validation (FR-7)
- Idempotency support for retries (FR-22)
- Group device listing with last locations (FR-9, FR-8)
- Last activity timestamp tracking (FR-11)
- Location retention policy enforcement (FR-12, FR-20)
- Request/response validation with detailed errors

**Acceptance Criteria**:
- Location uploads validate coordinate ranges (-90/90, -180/180)
- Batch uploads succeed atomically or fail completely
- Duplicate uploads with same idempotency key are deduplicated
- Group queries return last location for each active device
- Locations older than 30 days automatically deleted

---

#### Epic 4: Production Readiness & Operational Excellence
**Goal**: Harden system for production with observability, security, and deployment automation

**Business Value**: Ensures reliability, enables troubleshooting, supports multiple deployment tiers

**Story Count**: ~8 stories

**Key Capabilities**:
- Prometheus metrics export (FR-18)
- Rate limiting per API key (FR-16)
- API versioning (/api/v1/) (FR-25)
- Security headers and TLS configuration
- Background job scheduler (FR-20)
- Kubernetes deployment manifests
- Load testing and performance optimization
- Admin operations API (FR-26)
- Data privacy controls (export/deletion) (FR-24)

**Acceptance Criteria**:
- Prometheus scrapes metrics at /metrics endpoint
- Rate limit returns 429 with Retry-After header
- Background jobs run hourly for location cleanup
- Deployment supports zero-downtime rolling updates
- Performance meets NFR targets (<200ms p95, 10K concurrent)

---

**Epic Delivery Sequence**:
1. Epic 1 (Foundation) - Week 1-2
2. Epic 2 (Device Mgmt) - Week 2-3
3. Epic 3 (Location Tracking) - Week 3-5
4. Epic 4 (Production) - Week 5-6

**Total Estimated Stories**: ~33 stories across 4 epics

_Note: Detailed story breakdown with acceptance criteria will be generated in separate epics.md document_

## Out of Scope

The following features are intentionally excluded from the initial release (Epics 1-4) and preserved for future phases:

#### Phase 2+ Future Enhancements

1. **Location History Query API** (FR-13 placeholder) — Query historical locations for a device within retention window with date range filtering. Requires additional indexing and query optimization.

2. **Real-time Push Notifications** — While FCM tokens are stored, the actual push notification system for location updates, geofence alerts, or arrival notifications is not implemented. Mobile client will use polling for MVP.

3. **WebSocket Support** — Real-time bidirectional communication for live location streaming. MVP uses REST API with periodic polling.

4. **Geofencing / Location Alerts** — Automatic notifications when devices enter/exit defined geographic boundaries. Requires spatial query capabilities and alert rule engine.

5. **Location History Visualization** — Map-based visualization of location trails and patterns. Backend would need additional aggregation APIs.

6. **Multi-tenant API Keys** — Currently one API key per deployment. Multi-tenancy with isolated groups per organization not supported.

7. **OAuth/Social Authentication** — MVP uses API key auth only. User accounts, OAuth providers, and social login deferred.

8. **Advanced Analytics** — Usage statistics, location patterns, activity reports. Requires analytics data pipeline.

9. **Read Replicas for Scaling** — Database architecture supports it (NFR-9 mentions it), but deployment configuration and read/write splitting not implemented.

10. **Mobile SDK** — Official client library for Android/iOS to simplify integration. MVP provides REST API only.

#### Explicitly NOT Supported

- **iOS Client Support** — Backend is platform-agnostic, but initial focus is Android only
- **Web Dashboard** — No admin UI; operations via API or database tools only
- **Third-party Integrations** — No Zapier, IFTTT, or other integration hooks
- **Machine Learning** — No predictive location, anomaly detection, or behavior analysis
- **Blockchain/Decentralization** — Centralized architecture only

#### Technical Debt Items (Known Limitations)

- **In-memory Rate Limiting** — Current implementation doesn't work for multi-instance deployments. Requires Redis for production horizontal scaling.
- **Synchronous Last-Seen Updates** — Background updates (Story 2.6) may cause slight delays. Consider async batching for high-frequency clients.
- **No GraphQL Support** — REST only; GraphQL deferred to avoid complexity.
- **Limited Admin UI** — Admin operations via API only; no web interface.

---

## Assumptions and Dependencies

### Technical Assumptions

1. **Android Client Ready** — Mobile client is ready to integrate with backend API; no backend changes needed to support client requirements
2. **PostgreSQL 16 Available** — Deployment environments can provide PostgreSQL 16 or compatible version
3. **Rust 1.83+ Supported** — CI/CD and deployment environments support Rust Edition 2024
4. **SQLx Compile-Time Checks** — Development workflow includes `cargo sqlx prepare` for offline query verification
5. **Single-Region Deployment** — Initial deployment is single-region; no multi-region replication or geo-distribution required

### Infrastructure Dependencies

1. **PostgreSQL Database** — Requires PostgreSQL 16+ with uuid-ossp extension
2. **Container Runtime** — Docker for development, Kubernetes for production deployment (Tier 2-3)
3. **Load Balancer** — TLS termination handled by load balancer/ingress in production
4. **Monitoring Stack** — Prometheus + Grafana for metrics collection and visualization
5. **CI/CD Pipeline** — GitHub Actions or similar for automated build, test, deploy

### Business Assumptions

1. **Target Users Are Tech-Savvy Enough** — Users can install APK, share group IDs, and understand basic location sharing concepts
2. **Privacy Acceptable** — 30-day retention policy meets user expectations; no user has requested longer retention
3. **Self-Hosting Appeal** — Target market values self-hosting option over commercial SaaS convenience
4. **Group Size Sufficient** — 20 devices per group covers 95%+ of use cases (families, friend groups)
5. **Polling Acceptable for MVP** — Users will tolerate 30-60 second polling intervals instead of real-time push updates

### External Dependencies

1. **Firebase Cloud Messaging** — FCM tokens stored but notification system not implemented in MVP
2. **No Third-Party Services** — MVP has no external service dependencies beyond database

---

## Next Steps

### Phase 1: Architecture and Design (REQUIRED)

Since this is a Level 3 project, architecture planning must occur before implementation.

**1. Run Architecture Workflow**
   - **Command**: Start new chat with architect persona
   - **Input**: This PRD (`docs/PRD.md`), Epic structure (`docs/epics.md`), Technical spec (`docs/rust-backend-spec.md`)
   - **Output**: `docs/architecture.md`
   - **Focus Areas**:
     - Layered architecture (Routes → Services → Repositories → Entities)
     - Database schema review and optimization
     - Middleware stack design
     - Error handling patterns
     - Background job architecture

**2. Review and Align Technical Decisions**
   - Compare architecture.md with rust-backend-spec.md
   - Ensure consistency in technology choices
   - Update technical-decisions.md with any new architectural choices
   - Validate performance targets are achievable

**3. Optional: UX Specification**
   - If mobile client needs API design guidance
   - **Command**: `workflow plan-project` → select "UX specification"
   - **Output**: `docs/ux-specification.md`
   - Focus on API ergonomics, error responses, client-friendly data formats

---

### Phase 2: Detailed Planning

**1. Generate Detailed User Stories**
   - Expand each epic story with full acceptance criteria refinement
   - Add technical implementation notes from architecture.md
   - Prioritize stories for sprint planning

**2. Create Testing Strategy**
   - Unit test framework (cargo test)
   - Integration test approach (TestServer with test database)
   - Load testing plan (k6 scripts)
   - CI/CD test automation

**3. Define Success Metrics**
   - API latency targets (Prometheus dashboards)
   - Error rate thresholds
   - Database performance baselines
   - User adoption metrics (if applicable)

---

### Phase 3: Development Preparation

**1. Development Environment Setup**
   - Clone repository structure
   - Run `docker-compose up` for local development
   - Verify SQLx offline mode with `cargo sqlx prepare`
   - Configure IDE (VS Code + rust-analyzer)

**2. Sprint Planning**
   - Epic 1 (Foundation): Sprint 1-2 (2 weeks)
   - Epic 2 (Device Mgmt): Sprint 3 (1 week)
   - Epic 3 (Location Tracking): Sprint 4-5 (2 weeks)
   - Epic 4 (Production): Sprint 6 (1 week)

**3. CI/CD Pipeline**
   - GitHub Actions for build, test, lint
   - Automated `cargo clippy` and `cargo fmt` checks
   - Integration test suite runs on PR
   - Docker image builds and pushes to registry

---

### Immediate Next Action

**Start architecture workflow with the architect in a new context window**

Provide the following documents:
1. `docs/PRD.md` (this document)
2. `docs/epics.md` (epic breakdown)
3. `docs/rust-backend-spec.md` (technical specification)
4. `docs/project-workflow-analysis.md` (project analysis)

Ask the architect to:
- Create `docs/architecture.md` following the layered architecture pattern
- Define module boundaries and data flow
- Specify error handling and logging patterns
- Design background job scheduling system
- Validate that architecture supports all NFRs (performance, scalability, security)

---

## Document Status

- [ ] Goals and context validated with stakeholders
- [ ] All functional requirements reviewed
- [ ] User journeys cover all major personas
- [ ] Epic structure approved for phased delivery
- [ ] Ready for architecture phase

_Note: See technical-decisions.md for captured technical context_

---

_This PRD adapts to project level Level 3 (Full Product) - providing appropriate detail without overburden._
