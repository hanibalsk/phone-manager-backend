# Story 5.5: Movement Events Retrieval by Trip

**Epic**: Epic 5 - Movement Events API
**Status**: Blocked
**Created**: 2025-11-30

---

## User Story

**As a** mobile app
**I want** to retrieve all movement events for a specific trip
**So that** I can display complete trip visualization

## Prerequisites

- Story 5.2 (Create Movement Event Endpoint)
- **Epic 6 (Trips)** - BLOCKING: Trips table must exist before this endpoint can be implemented

## Acceptance Criteria

1. `GET /api/v1/trips/:tripId/movement-events` returns all events for trip
2. Query parameters: order (asc|desc, default asc for trip visualization)
3. Response: `{"events": [<movement-event-objects>], "count": <total>}`
4. Returns 404 if trip not found
5. Events sorted by timestamp in specified order
6. No pagination (trips typically <10K events)
7. Query executes in <200ms for 10K events

## Technical Notes

- Simple query on trip_id index (already created in Story 5.1)
- Consider adding limit parameter if trips grow very large

## Blocking Dependencies

This story cannot be implemented until Epic 6 (Trips) is complete because:
1. The trips table doesn't exist yet
2. Trip validation requires trip repository
3. GET endpoint would return 404 for all trips

## Implementation Tasks (for after Epic 6)

1. Create GetTripMovementEventsQuery struct
2. Create GetTripMovementEventsResponse struct
3. Add get_trip_movement_events route handler
4. Add route to API router
5. Write tests
