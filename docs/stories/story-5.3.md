# Story 5.3: Batch Movement Event Upload

**Epic**: Epic 5 - Movement Events API
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** mobile app
**I want** to upload multiple movement events at once
**So that** I can efficiently sync events when coming back online

## Prerequisites

- Story 5.2 (Create Movement Event Endpoint)

## Acceptance Criteria

1. `POST /api/v1/movement-events/batch` accepts JSON: `{"deviceId": "<uuid>", "events": [<movement-event-objects>]}`
2. Validates: 1-100 events per batch, max 2MB payload
3. Each event validated same as single upload (Story 5.2)
4. All events must belong to same deviceId
5. Optional tripId can differ per event in batch
6. Returns 400 if batch validation fails with details
7. Returns 404 if device not registered
8. Returns 200 with: `{"success": true, "processedCount": <count>}`
9. All events inserted in single transaction (atomic)
10. Request timeout: 30 seconds
11. Response time <500ms for 100 events

## Technical Notes

- Reuse validation logic from Story 5.2
- Use repository batch insert method (already exists)
- Configure 2MB body size limit in Axum

## Implementation Tasks

1. Create BatchMovementEventRequest DTO
2. Create BatchMovementEventResponse DTO
3. Add batch upload route handler
4. Configure body size limit
5. Add route to API router
6. Write tests
