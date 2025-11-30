# Project Workflow Analysis

**Date:** 2025-11-30
**Project:** phone-manager-backend (Movement Tracking Extension)
**Analyst:** Martin Janci

## Assessment Results

### Project Classification

- **Project Type:** Backend service/API (Rust Axum)
- **Project Level:** Level 3 (Full Product)
- **Instruction Set:** instructions-lg.md (Large project PRD workflow)

### Scope Summary

- **Brief Description:** Movement Tracking & Intelligent Path Detection feature extension for Phone Manager backend. Adds trip lifecycle management, movement event logging with sensor telemetry, enhanced location tracking with transportation mode context, and map-snapping path correction capabilities.
- **Estimated Stories:** 25-35 stories
- **Estimated Epics:** 4 epics (Movement Events, Trips, Enhanced Locations, Intelligent Path Detection)
- **Timeline:** 4-6 weeks development

### Context

- **Greenfield/Brownfield:** Brownfield - extending existing clean Rust codebase
- **Existing Documentation:** BACKEND_API_SPEC.md (API design), ANDROID_APP_SPEC.md (client spec), CLAUDE.md (project context)
- **Team Size:** Solo developer
- **Deployment Intent:** Production SaaS/application

## Recommended Workflow Path

### Primary Outputs

1. **PRD.md** — Product Requirements Document for Movement Tracking features
2. **epics.md** — Epic breakdown with user stories and acceptance criteria
3. **Architecture handoff** — Route to 3-solutioning workflow for architecture.md

### Workflow Sequence

1. ✅ Project Assessment (completed)
2. ✅ PRD Creation (completed) → PRD-movement-tracking.md
   - Product vision for movement tracking
   - User journeys for trip detection and history
   - Functional requirements (18 FRs)
   - Non-functional requirements (14 NFRs)
3. ✅ Epic Definition (completed) → epics.md (Epics 5-8)
   - Epic 5: Movement Events API (6 stories)
   - Epic 6: Trip Lifecycle Management (6 stories)
   - Epic 7: Enhanced Location Context (4 stories)
   - Epic 8: Intelligent Path Detection (6 stories)
4. ⏳ Architecture Handoff (pending)
   - PostGIS integration design
   - Map-snapping service integration
   - Background processing patterns

### Next Actions

1. Route to 3-solutioning workflow for architecture.md creation
2. Design PostGIS integration patterns for movement data
3. Select map-matching service (OSRM vs Valhalla vs Google Roads API)
4. Define background job architecture for path correction

## Special Considerations

- **Existing Infrastructure:** Leverages existing device registration, location tracking, geofence, and proximity alert systems already implemented in the backend.
- **PostGIS Integration:** New geospatial capabilities require PostgreSQL PostGIS extension for GEOGRAPHY types and spatial indexing.
- **Map-Snapping Backend:** Path correction requires integration with external service (OSRM, Valhalla, or Google Roads API).
- **Client Coordination:** Android app spec defines client-side trip detection algorithm, state machine, and UI screens.
- **Offline Support:** Client stores trips/events locally with sync status tracking.
- **Idempotency:** Trip creation uses client-generated localTripId for duplicate prevention on retries.

## Technical Preferences Captured

- **Language:** Rust 1.83+ (Edition 2024)
- **Framework:** Axum 0.8 with Tokio async runtime
- **Database:** PostgreSQL 16 with SQLx + PostGIS extension
- **New Tables:** movement_events, trips, frequent_locations, common_routes
- **Authentication:** Existing API key-based (SHA-256 hashed)
- **Serialization:** Serde with camelCase JSON (matching existing patterns)
- **Transportation Modes:** STATIONARY, WALKING, RUNNING, CYCLING, IN_VEHICLE, UNKNOWN
- **Detection Sources:** ACTIVITY_RECOGNITION, BLUETOOTH_CAR, ANDROID_AUTO, MULTIPLE, NONE

## Source Documents

| Document | Purpose |
|----------|---------|
| BACKEND_API_SPEC.md | Complete API specification with endpoints, data models, database schema |
| ANDROID_APP_SPEC.md | Client implementation spec with UI screens, state machine, database entities |
| CLAUDE.md | Existing project context and conventions |

---

_This analysis serves as the routing decision for the adaptive PRD workflow and will be referenced by future orchestration workflows._
