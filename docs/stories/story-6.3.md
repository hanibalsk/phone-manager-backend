# Story 6.3: Update Trip State

**Epic**: Epic 6 - Trip Lifecycle Management
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** mobile app
**I want** to update trip state to COMPLETED or CANCELLED
**So that** I can properly end trips

## Prerequisites

- Story 6.2 complete (create trip endpoint)

## Acceptance Criteria

1. `PATCH /api/v1/trips/:tripId` accepts JSON: `{"state": "<COMPLETED|CANCELLED>", "endTimestamp": <ms-epoch-optional>, "endLatitude": <float-optional>, "endLongitude": <float-optional>}`
2. State transitions: ACTIVE → COMPLETED, ACTIVE → CANCELLED only
3. Completed trips require endTimestamp, endLatitude, endLongitude
4. Cancelled trips don't require end location (trip may be invalid)
5. Returns 400 if invalid state transition (e.g., COMPLETED → ACTIVE)
6. Returns 404 if trip not found
7. Returns 200 with updated trip data
8. Triggers statistics calculation for COMPLETED trips (async) - deferred to Story 6.4
9. Updates updated_at timestamp

## Technical Notes

- State machine validation in domain layer (TripState::can_transition_to)
- Use tokio::spawn for async statistics calculation (Story 6.4)
- Consider event sourcing for complex state transitions (future)

## Implementation Tasks

- [x] Add PATCH /api/v1/trips/:tripId route handler
- [x] Implement state transition validation
- [x] Validate COMPLETED requires end location data
- [x] Return 400 for invalid transitions
- [x] Return 404 for trip not found
- [x] Return TripResponse with full trip data
- [x] Write unit tests
- [x] Update app.rs with new route

---

## Dev Notes

- Reuse TripState enum can_transition_to() method
- COMPLETED requires: endTimestamp, endLatitude, endLongitude
- CANCELLED doesn't require end location
- Statistics calculation deferred to Story 6.4

---

## Dev Agent Record

### Debug Log
- Starting Story 6.3 implementation

### Completion Notes
- Added PATCH /api/v1/trips/:tripId endpoint
- State transition validation using TripState::can_transition_to()
- COMPLETED requires endTimestamp, endLatitude, endLongitude
- CANCELLED can have optional end location
- Returns 400 for invalid state transitions
- Returns 404 if trip not found
- Full TripResponse with all fields returned
- All 225 API tests pass, clippy clean

---

## File List

- crates/api/src/routes/trips.rs (updated)
- crates/api/src/app.rs (updated)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed - all tasks done, tests pass |
