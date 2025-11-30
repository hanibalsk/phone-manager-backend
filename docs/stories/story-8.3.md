# Story 8.3: Automatic Path Correction on Trip Completion

**Epic**: Epic 8 - Intelligent Path Detection
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** backend system
**I want** to automatically correct paths when trips are completed
**So that** users get accurate movement data without manual intervention

## Prerequisites

- Story 8.1 complete (path correction schema)
- Story 8.2 complete (map-matching service)
- Story 7.x complete (trips infrastructure)

## Acceptance Criteria

1. On trip completion, extract location history as path
2. Call map-matching service with original path
3. Store corrected path in trip_path_corrections table
4. Record correction quality/confidence score
5. Handle failed corrections gracefully (set status to FAILED)
6. Skip correction if map-matching service is disabled (set status to SKIPPED)
7. Background job or event-driven trigger for path correction

## Technical Notes

- Use existing trip completion hook/event
- Extract locations ordered by timestamp
- Convert to coordinate array for map-matching
- Store original path (from locations) and corrected path (from service)
- Consider async processing via background job

## Implementation Tasks

- [x] Create PathCorrectionService in api crate
- [x] Extract trip locations as coordinate array
- [x] Convert locations to LINESTRING WKT for original_path
- [x] Call MapMatchingClient with coordinates
- [x] Convert matched coordinates to LINESTRING WKT for corrected_path
- [x] Store correction result in trip_path_corrections
- [x] Handle service disabled case (SKIPPED status)
- [x] Handle service failure case (FAILED status)
- [x] Add trip completion hook/trigger for path correction
- [ ] Add integration tests for path correction flow (deferred - requires running OSRM instance)

---

## Dev Notes

- PathCorrectionService orchestrates the correction workflow
- Needs access to TripRepository for location extraction
- Needs access to TripPathCorrectionRepository for storage
- Needs access to MapMatchingClient for correction
- Consider making async/background for performance

---

## Dev Agent Record

### Debug Log
- Starting Story 8.3 implementation
- Created PathCorrectionService with correct_trip_path() method
- Added get_locations_for_trip() to LocationRepository
- Integrated path correction into trip completion flow (update_trip_state)

### Completion Notes
- PathCorrectionService created with full workflow orchestration
- Supports three correction states: COMPLETED, FAILED, SKIPPED
- Handles insufficient locations (< 2 points) gracefully
- Integrated into trip completion via async tokio::spawn
- Path correction runs after trip statistics calculation
- Errors are logged but don't block trip completion

---

## File List

- `crates/api/src/services/path_correction.rs` - PathCorrectionService implementation
- `crates/api/src/services/mod.rs` - Export PathCorrectionService
- `crates/api/src/routes/trips.rs` - Added correct_trip_path() hook on completion
- `crates/api/src/main.rs` - Added services module
- `crates/persistence/src/repositories/location.rs` - Added get_locations_for_trip()

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed - PathCorrectionService with trip completion integration |
