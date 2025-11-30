# Story 5.4: Movement Events Retrieval by Device

**Epic**: Epic 5 - Movement Events API
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** mobile app
**I want** to retrieve movement events for a device
**So that** I can display movement history and analytics

## Prerequisites

- Story 5.2 (Create Movement Event Endpoint)

## Acceptance Criteria

1. `GET /api/v1/devices/:deviceId/movement-events` returns paginated events
2. Query parameters: cursor (string), limit (1-100, default 50), from (timestamp ms), to (timestamp ms), order (asc|desc, default desc)
3. Response: `{"events": [<movement-event-objects>], "pagination": {"nextCursor": "<cursor>", "hasMore": <bool>}}`
4. Each event includes: id, timestamp, latitude, longitude, accuracy, speed, bearing, altitude, transportationMode, confidence, detectionSource, tripId, createdAt
5. Returns 404 if device not found
6. Events sorted by timestamp in specified order
7. Cursor-based pagination uses (timestamp, id) for stable pagination
8. Query executes in <100ms for 100 events

## Technical Notes

- Use keyset pagination (timestamp, id) for efficiency
- Reuse MovementEventResponse DTO from Story 5.2
- Add query struct for pagination params

## Implementation Tasks

1. Create GetMovementEventsQuery struct
2. Create GetMovementEventsResponse struct
3. Add get_movement_events route handler
4. Add route to API router
5. Write tests
