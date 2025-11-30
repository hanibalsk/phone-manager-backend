# Story 5.2: Create Movement Event Endpoint

**Epic**: Epic 5 - Movement Events API
**Status**: Complete
**Created**: 2025-11-30

---

## User Story

**As a** mobile app
**I want** to upload a single movement event
**So that** I can record movement data with full sensor context

## Prerequisites

- Story 5.1 (Movement Event Database Schema)

## Acceptance Criteria

1. `POST /api/v1/movement-events` accepts JSON: `{"deviceId": "<uuid>", "tripId": "<uuid-optional>", "timestamp": <ms-epoch>, "latitude": <float>, "longitude": <float>, "accuracy": <float>, "speed": <float-optional>, "bearing": <float-optional>, "altitude": <float-optional>, "transportationMode": "<mode>", "confidence": <float>, "detectionSource": "<source>"}`
2. Validates: latitude (-90 to 90), longitude (-180 to 180), accuracy (>= 0), bearing (0-360 if present), speed (>= 0 if present), confidence (0.0-1.0), transportationMode (STATIONARY|WALKING|RUNNING|CYCLING|IN_VEHICLE|UNKNOWN), detectionSource (ACTIVITY_RECOGNITION|BLUETOOTH_CAR|ANDROID_AUTO|MULTIPLE|NONE)
3. Returns 400 for validation errors with field-level details
4. Returns 404 if device not registered
5. Returns 404 if tripId provided but trip doesn't exist
6. Returns 200 with: `{"id": "<uuid>", "createdAt": "<timestamp>"}`
7. Stores location as PostGIS GEOGRAPHY point
8. Response time <50ms for single event

## Technical Notes

- Domain model in `crates/domain/src/models/movement_event.rs`
- Repository in `crates/persistence/src/repositories/movement_event.rs`
- Use `ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)::geography` for PostGIS

## Implementation Tasks

1. Create TransportationMode and DetectionSource enums in domain layer
2. Create MovementEvent domain model
3. Create MovementEventRepository in persistence layer
4. Create request/response DTOs in API layer
5. Create movement_events route handler
6. Add route to API router
7. Write tests
