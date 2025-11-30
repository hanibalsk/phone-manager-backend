# Story 7.1: Location Schema Migration for Context Fields

**Epic**: Epic 7 - Enhanced Location Context
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** developer
**I want** to add transportation mode and detection source to locations
**So that** location data includes movement context

## Prerequisites

- Epic 3 complete (location tracking)

## Acceptance Criteria

1. ✅ Migration adds columns: transportation_mode (VARCHAR(20) nullable), detection_source (VARCHAR(30) nullable), trip_id (UUID nullable FK)
2. ✅ Foreign key to trips table with ON DELETE SET NULL
3. ✅ Existing location records retain NULL for new columns (backward compatible)
4. ✅ Index on trip_id for trip-based location queries
5. ✅ Migration is non-blocking (ALTER TABLE ADD COLUMN is fast in PostgreSQL)
6. ✅ Migration runs successfully without downtime

## Technical Notes

- Use ADD COLUMN with NULL default for non-blocking migration
- No need to backfill existing data
- FK constraint added after column creation

## Implementation Tasks

- [x] Create migration file for new location columns
- [x] Add transportation_mode VARCHAR(20) nullable
- [x] Add detection_source VARCHAR(30) nullable
- [x] Add trip_id UUID nullable with FK to trips
- [x] Add index on trip_id (partial index WHERE trip_id IS NOT NULL)
- [x] Update LocationEntity to include new fields
- [x] Update Location domain model with new fields
- [x] Update all repository SELECT queries to include new columns
- [x] Update test fixtures with new fields

---

## Dev Notes

- Locations table already exists from Epic 3
- New fields optional for backward compatibility
- FK with ON DELETE SET NULL preserves locations if trip deleted
- Partial index on trip_id only indexes non-null values for efficiency

---

## Dev Agent Record

### Debug Log
- Starting Story 7.1 implementation
- Created migration 013_location_context_fields.sql
- Updated LocationEntity with new fields
- Updated Location domain model with skip_serializing_if
- Updated From<LocationEntity> implementation
- Updated all SELECT queries in location repository
- Fixed test fixtures to include new fields
- All tests passing, clippy clean

### Completion Notes
Story 7.1 completed successfully. The migration adds three new context fields to the locations table:
- transportation_mode: VARCHAR(20) nullable for movement type (e.g., WALKING, IN_VEHICLE)
- detection_source: VARCHAR(30) nullable for how mode was detected (e.g., ACTIVITY_RECOGNITION)
- trip_id: UUID nullable FK to trips table with ON DELETE SET NULL

All changes are backward compatible - existing locations retain NULL for new columns.

---

## File List

- `crates/persistence/src/migrations/013_location_context_fields.sql` - New migration
- `crates/persistence/src/entities/location.rs` - Added context fields
- `crates/domain/src/models/location.rs` - Added context fields with skip_serializing_if
- `crates/persistence/src/repositories/location.rs` - Updated all SELECT queries

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

Story 7.1 implements a non-blocking database migration adding context fields to the locations table. All 6 acceptance criteria are met.

### Acceptance Criteria Coverage

| AC | Status | Evidence |
|----|--------|----------|
| Migration adds context columns | ✅ | `013_location_context_fields.sql:11-22` |
| FK to trips with ON DELETE SET NULL | ✅ | Preserves locations if trip deleted |
| Existing records retain NULL | ✅ | ADD COLUMN creates NULL defaults |
| Index on trip_id | ✅ | Partial index for efficiency |
| Migration is non-blocking | ✅ | ADD COLUMN is fast in PostgreSQL |
| No downtime required | ✅ | Non-blocking ALTER TABLE |

### Key Strengths

- Non-blocking migration pattern for production safety
- Partial index only indexes non-null trip_ids for efficiency
- Well-documented with SQL COMMENT statements
- Backward compatible with existing data

### Action Items

None required.
