# Story 6.2: Create Trip Endpoint with Idempotency

**Epic**: Epic 6 - Trip Lifecycle Management
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** mobile app
**I want** to create a trip with a client-generated ID
**So that** retries don't create duplicate trips

## Prerequisites

- Story 6.1 complete (trips table with schema)

## Acceptance Criteria

1. `POST /api/v1/trips` accepts JSON: `{"deviceId": "<uuid>", "localTripId": "<client-id>", "startTimestamp": <ms-epoch>, "startLatitude": <float>, "startLongitude": <float>, "transportationMode": "<mode>", "detectionSource": "<source>"}`
2. Validates all fields same as movement events
3. Returns 200 (not 201) if trip with same (deviceId, localTripId) exists - idempotent
4. Existing trip response includes all current data (may have been updated)
5. New trip created with state=ACTIVE
6. Returns 200/201 with: `{"id": "<uuid>", "localTripId": "<client-id>", "state": "ACTIVE", "startTimestamp": <ts>, "createdAt": "<timestamp>"}`
7. Returns 404 if device not registered
8. Returns 409 if device already has an ACTIVE trip with different localTripId
9. Only one ACTIVE trip allowed per device

## Technical Notes

- Use INSERT ... ON CONFLICT (device_id, local_trip_id) DO UPDATE for idempotency
- Check for existing ACTIVE trip before creating new one
- Transaction ensures atomic state check and creation

## Implementation Tasks

- [x] Create TripService in domain layer (logic in route handler for simplicity)
- [x] Create POST /api/v1/trips route handler
- [x] Add device existence validation
- [x] Add active trip conflict detection (409 if different localTripId)
- [x] Implement idempotent creation logic (200 for existing, 201 for new)
- [x] Add request validation (same as movement events)
- [x] Write unit tests for endpoint
- [x] Update routes/mod.rs with new routes
- [x] Update app.rs with new route

---

## Dev Notes

- Reuse TransportationMode and DetectionSource enums from movement_event
- Check device exists before creating trip (404)
- Check no other ACTIVE trip exists for device (409)
- Idempotent: same localTripId returns existing trip (200)

---

## Dev Agent Record

### Debug Log
- Starting Story 6.2 implementation

### Completion Notes
- Created POST /api/v1/trips endpoint in trips.rs route handler
- Validates device exists and is active (404 if not)
- Checks for existing ACTIVE trip with different localTripId (409 conflict)
- Returns 200 for idempotent retry (existing trip)
- Returns 201 for newly created trip
- All unit tests pass, clippy clean

---

## File List

- crates/api/src/routes/trips.rs (created)
- crates/api/src/routes/mod.rs (updated)
- crates/api/src/app.rs (updated)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-11-30 | Story created |
| 2025-11-30 | Story completed - all tasks done, tests pass |
