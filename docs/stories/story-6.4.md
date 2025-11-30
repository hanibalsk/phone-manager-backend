# Story 6.4: Trip Statistics Calculation

**Epic**: Epic 6 - Trip Lifecycle Management
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** backend system
**I want** to calculate trip statistics on completion
**So that** users see accurate trip summaries

## Prerequisites

- Story 6.3 complete (update trip state)
- Story 5.2 complete (movement events exist)

## Acceptance Criteria

1. Statistics calculated when trip state changes to COMPLETED
2. Calculates: distance_meters (sum of point-to-point distances), duration_seconds (end - start timestamp)
3. Distance calculated using Haversine formula on all trip movement events
4. Statistics stored in trips table
5. Calculation runs async (doesn't block API response)
6. Handles trips with 0-1 events (distance = 0)
7. Handles large trips (10K+ events) in <5 seconds
8. Failed calculation logged but doesn't affect trip state

## Technical Notes

- Use PostGIS ST_Distance for accurate geodetic distance
- Sum distances between consecutive points ordered by timestamp
- Consider background job queue for large trip processing
- Use tokio::spawn for async calculation

## Implementation Tasks

- [x] Create trip statistics service
- [x] Add function to calculate distance from movement events
- [x] Add function to calculate duration
- [x] Trigger calculation async on COMPLETED state change
- [x] Update trips table with calculated statistics
- [x] Handle edge cases (0-1 events)
- [x] Write unit tests
- [x] Integration test with movement events

---

## Dev Notes

- Movement events have trip_id FK linking them to trips
- Query movement events ordered by timestamp
- Use ST_Distance or Haversine formula for point-to-point distance
- Duration is simply end_timestamp - start_timestamp

---

## Dev Agent Record

### Debug Log
- Starting Story 6.4 implementation

### Completion Notes
- Added `get_events_for_trip()` method to MovementEventRepository
- Added `calculate_trip_distance()` using PostGIS ST_Distance with LAG window function
- Added `calculate_trip_statistics()` async function in trips.rs
- Modified `update_trip_state()` to trigger async calculation on COMPLETED
- Uses tokio::spawn for non-blocking execution
- Duration calculated as (end_timestamp - start_timestamp) / 1000
- Distance calculated using PostGIS for geodetic accuracy
- Edge case: 0-1 events returns distance = 0 via COALESCE
- Errors logged but don't affect trip state
- All 225+ tests pass, clippy clean

---

## File List

- crates/persistence/src/repositories/movement_event.rs (updated)
- crates/api/src/routes/trips.rs (updated)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed - async statistics calculation working |
