# Story 6.1: Trip Database Schema

**Epic**: Epic 6 - Trip Lifecycle Management
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** developer
**I want** a trips table with proper constraints and indexes
**So that** I can store trip data efficiently

## Prerequisites

- Epic 1 complete (database infrastructure)

## Acceptance Criteria

1. Migration creates `trips` table with columns: id (UUID PK), device_id (UUID FK), local_trip_id (VARCHAR(100)), state (VARCHAR(20)), start_timestamp (BIGINT), end_timestamp (BIGINT nullable), start_location (GEOGRAPHY(POINT, 4326)), end_location (GEOGRAPHY(POINT, 4326) nullable), transportation_mode (VARCHAR(20)), detection_source (VARCHAR(30)), distance_meters (DOUBLE PRECISION nullable), duration_seconds (BIGINT nullable), created_at (TIMESTAMPTZ), updated_at (TIMESTAMPTZ)
2. Unique constraint on (device_id, local_trip_id) for idempotency
3. Foreign key to devices with ON DELETE CASCADE
4. Index on (device_id, state) for active trip queries
5. Index on (device_id, start_timestamp DESC) for history queries
6. Check constraint: state IN ('ACTIVE', 'COMPLETED', 'CANCELLED')
7. Check constraint: distance_meters >= 0 when not null
8. Check constraint: duration_seconds >= 0 when not null
9. Trigger updates updated_at on row modification
10. Add FK constraint from movement_events.trip_id to trips.id with ON DELETE SET NULL

## Technical Notes

- local_trip_id from client ensures idempotent creation
- Use GEOGRAPHY for accurate distance calculations on completion
- This migration also adds the deferred FK from movement_events to trips

## Implementation Tasks

- [x] Create migration 012_trips.sql
- [x] Add trips table with all columns
- [x] Add unique constraint (device_id, local_trip_id)
- [x] Add FK to devices with CASCADE
- [x] Add all indexes
- [x] Add check constraints
- [x] Create updated_at trigger
- [x] Add FK from movement_events.trip_id to trips.id
- [x] Create TripEntity in persistence layer
- [x] Create TripRepository in persistence layer
- [x] Run migration and verify

---

## Dev Notes

- PostGIS already enabled from migration 011
- Reuse TransportationMode and DetectionSource from movement_events
- TripState enum: ACTIVE, COMPLETED, CANCELLED

---

## Dev Agent Record

### Debug Log
- Starting Story 6.1 implementation

### Completion Notes
- Created migration 012_trips.sql with complete trips table schema
- Added TripEntity with PostGIS GEOGRAPHY field handling
- Created Trip domain model with TripState enum (Active, Completed, Cancelled)
- Created CreateTripRequest, UpdateTripRequest DTOs with validation
- Created TripRepository with full CRUD operations
- Fixed pre-existing clippy issues (GeofenceEventType::from_str → parse, too_many_arguments)
- All 679 tests pass, clippy clean

---

## File List

- crates/persistence/src/migrations/012_trips.sql
- crates/persistence/src/entities/trip.rs
- crates/persistence/src/entities/mod.rs (updated)
- crates/domain/src/models/trip.rs
- crates/domain/src/models/mod.rs (updated)
- crates/persistence/src/repositories/trip.rs
- crates/persistence/src/repositories/mod.rs (updated)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed - all tasks done, tests pass |
| 2025-11-30 | Senior Developer Review: APPROVED |

---

## Senior Developer Review (AI)

**Reviewer**: Martin Janci
**Date**: 2025-11-30
**Outcome**: ✅ **APPROVED**

### Summary

Story 6.1 implements a comprehensive trips database schema with PostGIS geospatial support. All 10 acceptance criteria are met with proper constraints, indexes, and triggers.

### Acceptance Criteria Coverage

| AC | Status | Evidence |
|----|--------|----------|
| trips table with all columns | ✅ | `012_trips.sql:6-20` |
| Unique constraint (device_id, local_trip_id) | ✅ | `012_trips.sql:23` |
| FK to devices with CASCADE | ✅ | `012_trips.sql:8` |
| Index (device_id, state) | ✅ | `012_trips.sql:42` |
| Index (device_id, start_timestamp DESC) | ✅ | `012_trips.sql:45` |
| Check constraint on state | ✅ | `012_trips.sql:26` |
| Check constraint distance_meters >= 0 | ✅ | `012_trips.sql:29` |
| Check constraint duration_seconds >= 0 | ✅ | `012_trips.sql:32` |
| updated_at trigger | ✅ | `012_trips.sql:51-63` |
| FK from movement_events.trip_id | ✅ | `012_trips.sql:67-69` |

### Key Strengths

- Proper PostGIS GEOGRAPHY(POINT, 4326) for accurate geospatial calculations
- Idempotency via unique constraint on (device_id, local_trip_id)
- Comprehensive index strategy for common query patterns
- Well-documented schema with COMMENTs
- Clean entity-to-domain conversion with proper error handling

### Action Items

None required.
