# Story 5.1: Movement Event Database Schema

**Epic**: Epic 5 - Movement Events API
**Status**: Complete
**Created**: 2025-11-30
**Completed**: 2025-11-30

---

## User Story

**As a** developer
**I want** a PostGIS-enabled movement_events table
**So that** I can store movement data with geospatial capabilities

## Prerequisites

- Epic 1 complete (database infrastructure)

## Acceptance Criteria

1. Migration creates `movement_events` table with columns: id (UUID), device_id (UUID FK), trip_id (UUID FK nullable), timestamp (BIGINT), location (GEOGRAPHY(POINT, 4326)), accuracy (REAL), speed (REAL nullable), bearing (REAL nullable), altitude (DOUBLE PRECISION nullable), transportation_mode (VARCHAR), confidence (REAL), detection_source (VARCHAR), created_at (TIMESTAMPTZ)
2. Foreign key constraint to devices table with ON DELETE CASCADE
3. Foreign key constraint to trips table with ON DELETE SET NULL
4. Index on (device_id, timestamp) for efficient queries
5. Index on (trip_id) for trip-based lookups
6. PostGIS GIST index on location column
7. Check constraint: confidence BETWEEN 0.0 AND 1.0
8. Check constraint: accuracy >= 0
9. Migration runs successfully with `sqlx migrate run`

## Technical Notes

- Use GEOGRAPHY type for accurate distance calculations
- SRID 4326 (WGS84) for GPS coordinates
- Store timestamp as milliseconds epoch for client compatibility

## Implementation Details

### Migration File
- Create migration 006: movement_events table with PostGIS support
- First enable PostGIS extension if not already enabled
- Create the movement_events table with all specified columns
- Add foreign key constraints with appropriate ON DELETE behavior
- Create indexes for query optimization

### Database Design
- Use GEOGRAPHY(POINT, 4326) for location storage (WGS84 coordinate system)
- BIGINT for timestamp to store milliseconds since epoch
- VARCHAR for transportation_mode and detection_source for flexibility
- REAL for accuracy, speed, bearing (sufficient precision for GPS data)
- DOUBLE PRECISION for altitude (higher precision needed)

---

## Testing Requirements

1. Migration applies successfully
2. Table exists with all specified columns and correct types
3. Foreign key constraints work correctly (CASCADE for devices, SET NULL for trips)
4. Indexes created and queryable
5. Check constraints prevent invalid data
6. PostGIS functions work on location column

---

## Implementation Notes

### Completed Items
- Created migration `011_movement_events.sql`
- Enabled PostGIS extension
- Created `movement_events` table with all required columns
- Added all check constraints for data validation
- Created all required indexes including GIST index for PostGIS
- FK to devices table with ON DELETE CASCADE
- trip_id column ready for FK constraint (added in Story 6.1 when trips table created)

### Verification
- Migration applied successfully to Supabase PostgreSQL
- PostGIS 3.3 confirmed active
- Table structure verified with \d command
- All indexes and constraints in place

---

## Senior Developer Review

**Reviewer**: Senior Developer Review Workflow
**Review Date**: 2025-11-30
**Outcome**: ✅ APPROVED

### Summary
Database schema implementation for movement events with PostGIS support is complete and well-designed. The migration correctly enables PostGIS, creates the movement_events table with proper column types, and establishes all required indexes and constraints.

### Key Findings

**Strengths**:
- ✅ Correct use of GEOGRAPHY(POINT, 4326) for WGS84 GPS coordinates
- ✅ Comprehensive check constraints (confidence 0-1, accuracy ≥0, bearing 0-360, latitude/longitude bounds)
- ✅ GIST spatial index for efficient geospatial queries
- ✅ Composite index on (device_id, timestamp) for optimal pagination
- ✅ Proper ON DELETE CASCADE for device_id foreign key
- ✅ VARCHAR for enums provides flexibility for future values

**No Critical/High Issues Found**

### Acceptance Criteria Coverage
| # | Criterion | Status |
|---|-----------|--------|
| 1 | Table with all columns | ✅ Met |
| 2 | FK to devices with CASCADE | ✅ Met |
| 3 | FK to trips with SET NULL | ⏳ Deferred (Story 6.1) |
| 4 | Index on (device_id, timestamp) | ✅ Met |
| 5 | Index on trip_id | ✅ Met |
| 6 | PostGIS GIST index | ✅ Met |
| 7 | Confidence constraint | ✅ Met |
| 8 | Accuracy constraint | ✅ Met |
| 9 | Migration runs successfully | ✅ Met |

### Architectural Alignment
Follows project layered architecture. Migration location in `crates/persistence/src/migrations/` aligns with CLAUDE.md specification. PostGIS usage consistent with existing location tracking pattern.

### Security Notes
- No security concerns identified
- Proper use of parameterized queries enforced by SQLx

### Best Practices
- Migration file follows sequential numbering (011)
- Check constraints enforce data integrity at database level
- Geographic data properly normalized using WGS84 standard
