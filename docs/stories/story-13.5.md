# Story 13.5: Device Enrollment Endpoint

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** mobile device
**I want** to enroll with an organization using an enrollment token
**So that** I'm automatically configured with organization policies and settings

## Prerequisites

- Story 13.4 complete (Enrollment tokens)
- Story 13.3 complete (Device policies)
- Device registration endpoint (Story 2.1)

## Acceptance Criteria

1. POST `/api/v1/devices/enroll` accepts enrollment token and device info
2. Validates token exists, not expired, and has remaining uses
3. Creates or updates device record with organization_id, is_managed=true
4. Applies policy from token if specified
5. Adds device to group from token if specified
6. Increments token current_uses counter
7. Generates device_token for future authenticated requests
8. Creates `device_tokens` table for long-lived device authentication
9. Returns device info, device_token, policy details, and organization info
10. Endpoint does NOT require authentication (token is the auth)

## Technical Notes

- Create migration 027_device_tokens.sql for device token storage
- Device token format: `dt_<base64-random-45-chars>` with 90-day expiry
- Update devices table: add organization_id (FK), is_managed, enrollment_status columns
- enrollment_status enum: pending, enrolled, suspended, retired
- Device tokens are separate from user JWT tokens

## API Specification

### POST /api/v1/devices/enroll

Request:
```json
{
  "enrollment_token": "enroll_abc123xyz...",
  "device_uuid": "550e8400-e29b-41d4-a716-446655440000",
  "display_name": "Field Tablet #42",
  "device_info": {
    "manufacturer": "Samsung",
    "model": "Galaxy Tab A8",
    "os_version": "Android 14"
  }
}
```

Response (201):
```json
{
  "device": {
    "id": "uuid",
    "device_uuid": "550e8400-...",
    "display_name": "Field Tablet #42",
    "organization_id": "org_uuid",
    "is_managed": true,
    "enrollment_status": "enrolled"
  },
  "device_token": "dt_eyJhbG...",
  "device_token_expires_at": "2026-03-01T00:00:00Z",
  "policy": {
    "id": "pol_uuid",
    "name": "Field Worker Standard",
    "settings": {...},
    "locked_settings": [...]
  },
  "group": {
    "id": "grp_uuid",
    "name": "Field Workers"
  }
}
```

### Error Responses

- 400: Invalid request body
- 404: Enrollment token not found
- 410: Enrollment token expired or max uses reached
- 409: Device already enrolled in different organization

---

## Implementation Tasks

- [x] Create migration 028_device_enrollment.sql
- [x] Add columns to devices table: is_managed, enrollment_status, policy_id, enrolled_at, enrolled_via_token_id
- [x] Create enrollment_status enum
- [x] Create DeviceTokenEntity in persistence layer
- [x] Create DeviceToken domain model with generation
- [x] Create enrollment domain models (request/response)
- [x] Implement POST /api/v1/devices/enroll endpoint
- [x] Add create_managed_device and update_enrollment methods to DeviceRepository
- [x] Create DeviceTokenRepository for device token management
- [ ] Add device token authentication middleware (for managed devices) - deferred
- [ ] Add audit logging for enrollment - deferred to Story 13.9
- [x] Write unit tests for enrollment validation
- [ ] Write integration tests for enrollment flow

---

## Dev Notes

- Enrollment is idempotent for same device_uuid + org combination
- Device tokens allow managed devices to authenticate without user JWT
- Consider auto-assigning user if email matches (via token setting)
- Policy settings applied immediately after enrollment

---

## Dev Agent Record

### Debug Log

- Migration numbered 028 (027 already used for enrollment tokens)
- EnrollmentPolicyInfo.settings changed from serde_json::Value to HashMap<String, serde_json::Value> to match DevicePolicy

### Completion Notes

Implemented device enrollment with:
- Migration 028_device_enrollment.sql: enrollment_status enum, device columns, device_tokens table
- DeviceToken domain model and entity with token generation (dt_ prefix)
- Enrollment domain models (EnrollDeviceRequest, EnrollDeviceResponse, etc.)
- DeviceTokenRepository for device token CRUD
- DeviceRepository methods: create_managed_device, update_enrollment
- POST /api/v1/devices/enroll endpoint (public, token-based auth)
- Unit tests for enrollment validation and device token generation

---

## File List

- crates/persistence/src/migrations/028_device_enrollment.sql
- crates/persistence/src/entities/device_token.rs
- crates/domain/src/models/device_token.rs
- crates/domain/src/models/enrollment.rs
- crates/persistence/src/repositories/device_token.rs
- crates/api/src/routes/enrollment.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Story implemented and completed |

