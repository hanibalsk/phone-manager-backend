# Story 8.1: Path Correction Database Schema

**Epic**: Epic 8 - Intelligent Path Detection
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** developer
**I want** to store corrected path coordinates
**So that** users can view map-snapped trip paths

## Prerequisites

- Story 6.1 complete (trips table)

## Acceptance Criteria

1. Migration creates `trip_path_corrections` table: id (UUID), trip_id (UUID FK UNIQUE), original_path (GEOGRAPHY(LINESTRING, 4326)), corrected_path (GEOGRAPHY(LINESTRING, 4326) nullable), correction_quality (REAL nullable), correction_status (VARCHAR(20)), created_at (TIMESTAMPTZ), updated_at (TIMESTAMPTZ)
2. One-to-one relationship with trips (trip_id UNIQUE)
3. correction_status enum: PENDING, COMPLETED, FAILED, SKIPPED
4. correction_quality range: 0.0-1.0 (confidence metric)
5. Foreign key with ON DELETE CASCADE
6. Index on (correction_status) for processing queries

## Technical Notes

- Use LINESTRING to store ordered path points
- Store original for comparison and fallback
- Quality metric from map-matching service confidence

## Implementation Tasks

- [x] Create migration file for trip_path_corrections table
- [x] Add id UUID primary key with default gen_random_uuid()
- [x] Add trip_id UUID with UNIQUE constraint and FK to trips
- [x] Add original_path GEOGRAPHY(LINESTRING, 4326)
- [x] Add corrected_path GEOGRAPHY(LINESTRING, 4326) nullable
- [x] Add correction_quality REAL nullable with CHECK 0.0-1.0
- [x] Add correction_status VARCHAR(20) NOT NULL
- [x] Add created_at and updated_at TIMESTAMPTZ
- [x] Add FK constraint with ON DELETE CASCADE
- [x] Add index on correction_status
- [x] Create TripPathCorrectionEntity struct
- [x] Create TripPathCorrection domain model
- [x] Create TripPathCorrectionRepository with basic CRUD

---

## Dev Notes

- PostgreSQL PostGIS extension required for GEOGRAPHY type
- LINESTRING represents ordered sequence of points
- One-to-one with trips via UNIQUE constraint on trip_id
- Status values: PENDING, COMPLETED, FAILED, SKIPPED

---

## Dev Agent Record

### Debug Log
- Starting Story 8.1 implementation
- Verified PostGIS extension already available (trips table uses GEOGRAPHY type)
- Created migration 014_trip_path_corrections.sql
- Created TripPathCorrectionEntity with GeoJSON path representation
- Created TripPathCorrection domain model with CorrectionStatus enum
- Created TripPathCorrectionRepository with CRUD operations
- All tests passing, clippy clean

### Completion Notes
Story 8.1 completed successfully. Created trip_path_corrections table for storing:
- original_path: GEOGRAPHY(LINESTRING) from GPS traces
- corrected_path: GEOGRAPHY(LINESTRING) from map-matching service
- correction_quality: Confidence metric 0.0-1.0
- correction_status: PENDING, COMPLETED, FAILED, SKIPPED

One-to-one relationship with trips via UNIQUE constraint on trip_id.
Repository handles WKT/GeoJSON conversions for PostGIS operations.

---

## File List

- `crates/persistence/src/migrations/014_trip_path_corrections.sql` - New migration
- `crates/persistence/src/entities/trip_path_correction.rs` - Entity struct
- `crates/domain/src/models/trip_path_correction.rs` - Domain model and DTOs
- `crates/persistence/src/repositories/trip_path_correction.rs` - Repository CRUD
- `crates/domain/src/models/mod.rs` - Added module export
- `crates/persistence/src/entities/mod.rs` - Added entity export
- `crates/persistence/src/repositories/mod.rs` - Added repository export

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed |
| 2025-11-30 | Senior Developer Review: APPROVED |

---

## Senior Developer Review (AI)

**Reviewer**: Martin Janci
**Date**: 2025-11-30
**Outcome**: ✅ **APPROVED**

### Summary

Story 8.1 implements the path correction database schema with proper PostGIS GEOGRAPHY types. All 6 acceptance criteria are met.

### Acceptance Criteria Coverage

| AC | Status | Evidence |
|----|--------|----------|
| Migration creates table with all columns | ✅ | `014_trip_path_corrections.sql:6-25` |
| One-to-one relationship (trip_id UNIQUE) | ✅ | UNIQUE constraint on trip_id |
| correction_status enum values | ✅ | CHECK constraint `IN ('PENDING', 'COMPLETED', 'FAILED', 'SKIPPED')` |
| correction_quality range 0.0-1.0 | ✅ | CHECK constraint `chk_correction_quality` |
| Foreign key with ON DELETE CASCADE | ✅ | `REFERENCES trips(id) ON DELETE CASCADE` |
| Index on correction_status | ✅ | `idx_trip_path_corrections_status` |

### Key Strengths

- Uses GEOGRAPHY(LINESTRING, 4326) for accurate geospatial storage
- Repository handles WKT/GeoJSON conversions transparently
- CorrectionStatus enum with FromStr/Display implementations
- updated_at trigger for automatic timestamp management
- Comprehensive test coverage for entity-to-domain conversion

### Action Items

None required.
