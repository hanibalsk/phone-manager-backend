# Story 15.2: Webhook Event Delivery

**Epic**: Epic 15 - Webhook System
**Status**: Completed
**Created**: 2025-12-06

---

## Story

As a **backend system**,
I want **to deliver webhook notifications when events occur**,
So that **external systems receive real-time updates**.

## Prerequisites

- Story 15.1 complete (Webhook Registration and Management API)
- Geofences CRUD implemented (existing)

## Background

The frontend mobile app (phone-manager) already sends geofence events to the backend when geofence transitions occur. The app expects:
- `POST /api/v1/geofence-events` - Create geofence event
- `GET /api/v1/geofence-events?deviceId={id}` - List events
- `GET /api/v1/geofence-events/{eventId}` - Get single event

The response includes `webhookDelivered` and `webhookResponseCode` fields to indicate webhook delivery status.

[Source: phone-manager/app/src/main/java/three/two/bit/phonemanager/network/GeofenceEventApiService.kt]
[Source: phone-manager/app/src/main/java/three/two/bit/phonemanager/network/models/GeofenceEventModels.kt]

## Acceptance Criteria

### AC 15.2.1: Geofence Events Database Schema
1. Migration creates `geofence_events` table with columns:
   - `id` (BIGSERIAL PK)
   - `event_id` (UUID NOT NULL UNIQUE)
   - `device_id` (UUID FK to devices)
   - `geofence_id` (UUID FK to geofences)
   - `event_type` (VARCHAR(20) NOT NULL) - enter, exit, dwell
   - `timestamp` (BIGINT NOT NULL) - milliseconds epoch
   - `latitude` (DOUBLE PRECISION NOT NULL)
   - `longitude` (DOUBLE PRECISION NOT NULL)
   - `webhook_delivered` (BOOLEAN DEFAULT false)
   - `webhook_response_code` (INTEGER NULL)
   - `created_at` (TIMESTAMPTZ)
2. Foreign key to devices with ON DELETE CASCADE
3. Foreign key to geofences with ON DELETE CASCADE
4. Index on `device_id` for efficient lookups
5. Index on `(device_id, timestamp DESC)` for time-ordered queries

### AC 15.2.2: Create Geofence Event Endpoint
1. `POST /api/v1/geofence-events` accepts JSON:
   ```json
   {
     "device_id": "<uuid>",
     "geofence_id": "<uuid>",
     "event_type": "enter|exit|dwell",
     "timestamp": <milliseconds-epoch>,
     "latitude": <float>,
     "longitude": <float>
   }
   ```
2. Validates:
   - `device_id`: must exist and be active
   - `geofence_id`: must exist and belong to device
   - `event_type`: one of enter, exit, dwell
   - `timestamp`: positive integer
   - `latitude`: -90 to 90
   - `longitude`: -180 to 180
3. Generates unique `event_id` (UUID)
4. Returns 201 Created with event object including webhook status
5. Returns 400 for validation errors
6. Returns 404 if device or geofence not found

### AC 15.2.3: List Geofence Events Endpoint
1. `GET /api/v1/geofence-events?deviceId=<uuid>` returns:
   ```json
   {
     "events": [<event-objects>],
     "total": <count>
   }
   ```
2. Optional query parameters: `geofenceId`, `limit` (default 50, max 100)
3. Returns 400 if `deviceId` query parameter missing
4. Sorted by `timestamp` DESC (newest first)

### AC 15.2.4: Get Geofence Event Endpoint
1. `GET /api/v1/geofence-events/:eventId` returns single event object
2. Returns 404 if event not found
3. Response includes webhook delivery status

### AC 15.2.5: Webhook Delivery Service
1. When geofence event is created, trigger webhook delivery asynchronously
2. Find all enabled webhooks for the device (`owner_device_id`)
3. For each webhook, deliver payload:
   ```json
   {
     "event_type": "geofence_enter|geofence_exit|geofence_dwell",
     "device_id": "<uuid>",
     "geofence_id": "<uuid>",
     "geofence_name": "<string>",
     "timestamp": <milliseconds>,
     "location": {
       "latitude": <float>,
       "longitude": <float>
     }
   }
   ```
4. Sign payload with HMAC-SHA256 using webhook secret
5. Include signature in `X-Webhook-Signature` header
6. Use 5-second timeout per delivery attempt
7. Update event's `webhook_delivered` and `webhook_response_code` after delivery

### AC 15.2.6: Async Delivery (Non-Blocking)
1. Webhook delivery does not block the API response
2. Use `tokio::spawn` for async delivery
3. Event created and returned immediately
4. Webhook delivery happens in background
5. Event record updated after delivery completes

## Tasks / Subtasks

- [ ] Task 1: Database Schema (AC: 15.2.1)
  - [ ] Create migration file `034_geofence_events.sql`
  - [ ] Add foreign key constraints
  - [ ] Add indexes
  - [ ] Run `sqlx migrate run` and verify

- [ ] Task 2: Geofence Event Entity and Repository (AC: 15.2.2-15.2.4)
  - [ ] Create `GeofenceEventEntity` in `crates/persistence/src/entities/geofence_event.rs`
  - [ ] Create `GeofenceEventRepository` in `crates/persistence/src/repositories/geofence_event.rs`
  - [ ] Create domain model in `crates/domain/src/models/geofence_event.rs`
  - [ ] Add CRUD operations: create, find_by_event_id, find_by_device_id, update_webhook_status

- [ ] Task 3: Geofence Events Endpoints (AC: 15.2.2-15.2.4)
  - [ ] Create `crates/api/src/routes/geofence_events.rs`
  - [ ] Implement `POST /api/v1/geofence-events` handler
  - [ ] Implement `GET /api/v1/geofence-events` handler (list)
  - [ ] Implement `GET /api/v1/geofence-events/:eventId` handler
  - [ ] Add request/response DTOs with validation
  - [ ] Register routes in `app.rs`

- [ ] Task 4: Webhook Delivery Service (AC: 15.2.5-15.2.6)
  - [ ] Create `crates/api/src/services/webhook_delivery.rs`
  - [ ] Implement HMAC-SHA256 signature generation
  - [ ] Implement async HTTP delivery with timeout
  - [ ] Implement payload formatting for geofence events
  - [ ] Update event webhook_delivered status after delivery

- [ ] Task 5: Integration (AC: 15.2.5-15.2.6)
  - [ ] Wire webhook delivery service into create_geofence_event handler
  - [ ] Use `tokio::spawn` for non-blocking delivery
  - [ ] Handle delivery errors gracefully (log, don't fail request)

- [ ] Task 6: Testing
  - [ ] Run `cargo test --workspace`
  - [ ] Run `cargo clippy --workspace -- -D warnings`
  - [ ] Write unit tests for webhook signature generation
  - [ ] Write unit tests for geofence event validation

## Dev Notes

### Frontend Alignment

The mobile app expects these exact API contracts:

**Request DTO** (from `GeofenceEventModels.kt`):
```kotlin
CreateGeofenceEventRequest(
    deviceId: String,      // snake_case: device_id
    geofenceId: String,    // snake_case: geofence_id
    eventType: GeofenceEventType,  // enum: ENTER, EXIT, DWELL
    timestamp: String,     // ISO 8601 format
    latitude: Double,
    longitude: Double
)
```

**Response DTO** (from `GeofenceEventModels.kt`):
```kotlin
GeofenceEventDto(
    eventId: String,       // snake_case: event_id
    deviceId: String,      // snake_case: device_id
    geofenceId: String,    // snake_case: geofence_id
    geofenceName: String?, // snake_case: geofence_name
    eventType: GeofenceEventType,
    timestamp: String,
    latitude: Double,
    longitude: Double,
    webhookDelivered: Boolean,     // snake_case: webhook_delivered
    webhookResponseCode: Int?      // snake_case: webhook_response_code
)
```

### Webhook Payload Format

The webhook payload sent to external systems should match what automation platforms expect:

```json
{
  "event_type": "geofence_enter",
  "device_id": "550e8400-e29b-41d4-a716-446655440000",
  "geofence_id": "660e8400-e29b-41d4-a716-446655440001",
  "geofence_name": "Home",
  "timestamp": 1701878400000,
  "location": {
    "latitude": 37.7749,
    "longitude": -122.4194
  }
}
```

### HMAC-SHA256 Signature

Generate signature using webhook secret:
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?;
mac.update(payload.as_bytes());
let signature = hex::encode(mac.finalize().into_bytes());
```

Header: `X-Webhook-Signature: sha256=<hex-signature>`

### Architecture Patterns

- Follow existing layered architecture: Routes → Services → Repositories → Entities
- Use `validator` crate for request validation
- Use SQLx compile-time checked queries
- Use `tokio::spawn` for async webhook delivery
- Use `reqwest` for HTTP client

### Security Considerations

- Validate device_id and geofence_id ownership
- HMAC signature prevents payload tampering
- 5-second timeout prevents hanging on slow webhooks
- No sensitive data in webhook payload

## Dev Agent Record

### Context Reference

### Agent Model Used

Claude claude-opus-4-5-20251101

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2025-12-06 | Claude | Initial story creation - Webhook Event Delivery |
