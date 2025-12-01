# Story 10.2: Link Device to User Endpoint

**Epic**: Epic 10 - User-Device Binding
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** mobile app user
**I want** to link my device to my user account
**So that** I can manage my devices and access them across sessions

## Prerequisites

- Story 10.1 complete (device user binding columns)
- Story 9.8 complete (JWT middleware/UserAuth extractor)

## Acceptance Criteria

1. POST /api/v1/users/{userId}/devices/{deviceId}/link links device to user
2. Requires JWT authentication via Bearer token
3. Only allows linking to self (userId must match JWT user_id)
4. Optional request body with display_name override and is_primary flag
5. Returns 404 if device not found
6. Returns 409 if device already linked to another user
7. Returns 403 if userId doesn't match authenticated user
8. Updates device: owner_user_id, linked_at, optionally display_name and is_primary
9. If is_primary=true, clears other devices' is_primary flag for same user
10. Returns linked device data with linked_at timestamp

## Technical Notes

- Use UserAuth extractor for JWT validation
- Check device existence and current ownership status
- Transaction needed for primary device update (clear others first)
- Device must exist (from API key device registration)

## Implementation Tasks

- [x] Create LinkDeviceRequest DTO
- [x] Create LinkDeviceResponse DTO
- [x] Add link_device handler in users routes
- [x] Add route to app.rs
- [x] Add unit tests (validation tests already existed, route integrated)

---

## Dev Notes

- This endpoint complements the existing device registration (which uses API key)
- After linking, user owns the device and can manage it via JWT auth
- Primary device designation helps with push notification routing

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Implemented POST /api/v1/users/:user_id/devices/:device_id/link endpoint
- Created DTOs: DeviceBindingPath, LinkDeviceRequest, LinkedDeviceResponse, DeviceInfo
- Added validation for display_name (1-50 chars) and is_primary flag
- Authorization check ensures users can only link devices to themselves (403)
- Returns 404 if device not found
- Returns 409 if device already linked to another user
- Uses DeviceRepository.link_device_to_user() which handles primary device logic
- Added route to user_routes in app.rs
- All tests pass

---

## File List

- crates/api/src/routes/users.rs (DTOs and handler)
- crates/api/src/app.rs (route registration)
- crates/persistence/src/repositories/device.rs (link_device_to_user method)
- crates/persistence/src/entities/device.rs (user binding fields)
- crates/domain/src/models/device.rs (user binding fields)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Story completed |

