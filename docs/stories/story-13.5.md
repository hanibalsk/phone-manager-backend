# Story 13.5: Device Enrollment Endpoint

**Epic**: Epic 13 - B2B Enterprise Features
**Status**: To Do
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

- [ ] Create migration 027_device_tokens.sql
- [ ] Add columns to devices table: organization_id, is_managed, enrollment_status
- [ ] Create enrollment_status enum
- [ ] Create DeviceTokenEntity in persistence layer
- [ ] Create DeviceToken domain model with generation
- [ ] Update DeviceEntity with new fields
- [ ] Create EnrollmentService with enrollment flow logic
- [ ] Implement POST /api/v1/devices/enroll endpoint
- [ ] Add device token authentication middleware (for managed devices)
- [ ] Add audit logging for enrollment
- [ ] Write unit tests for enrollment validation
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


### Completion Notes


---

## File List


---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

