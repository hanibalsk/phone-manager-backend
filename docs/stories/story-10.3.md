# Story 10.3: List User's Devices Endpoint

**Epic**: Epic 10 - User-Device Binding
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** mobile app user
**I want** to see all devices linked to my account
**So that** I can manage my devices and see their status

## Prerequisites

- Story 10.1 complete (device user binding columns)
- Story 10.2 complete (link device endpoint)
- Story 9.8 complete (JWT middleware/UserAuth extractor)

## Acceptance Criteria

1. GET /api/v1/users/{userId}/devices lists devices owned by user
2. Requires JWT authentication via Bearer token
3. Only allows listing own devices (userId must match JWT user_id)
4. Returns 403 if userId doesn't match authenticated user
5. Returns array of devices with id, device_uuid, display_name, is_primary, linked_at, last_seen_at
6. Supports optional query parameters: include_inactive (default: false)
7. Orders devices by is_primary DESC, linked_at DESC (primary first, then by link date)
8. Returns empty array if user has no devices

## Technical Notes

- Use UserAuth extractor for JWT validation
- Leverage DeviceRepository.find_devices_by_user() method (already implemented in Story 10.2)
- Transform DeviceEntity to UserDeviceResponse DTO

## Implementation Tasks

- [x] Create UserDevicesPath struct for path extraction
- [x] Create UserDeviceResponse DTO
- [x] Create ListUserDevicesQuery for optional parameters
- [x] Add list_user_devices handler in users routes
- [x] Add route to app.rs

---

## Dev Notes

- Complements link_device endpoint
- Primary device appears first in list
- Can optionally include inactive devices for device management

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Implemented GET /api/v1/users/:user_id/devices endpoint
- Created DTOs: UserDevicesPath, ListUserDevicesQuery, UserDeviceResponse, ListUserDevicesResponse
- Supports include_inactive query parameter (default: false)
- Returns devices ordered by is_primary DESC, linked_at DESC
- Authorization check ensures users can only list their own devices (403)
- Response includes device_uuid, display_name, platform, is_primary, active, linked_at, last_seen_at
- All tests pass

---

## File List

- crates/api/src/routes/users.rs (DTOs and handler)
- crates/api/src/app.rs (route registration)
- crates/persistence/src/repositories/device.rs (updated ordering)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Story completed |

