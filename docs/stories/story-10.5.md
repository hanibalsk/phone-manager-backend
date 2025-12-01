# Story 10.5: Transfer Device Ownership Endpoint

**Epic**: Epic 10 - User-Device Binding
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** device owner
**I want** to transfer my device to another user
**So that** I can give a device to someone else while maintaining its data

## Prerequisites

- Story 10.1 complete (device user binding columns)
- Story 10.2 complete (link device endpoint)
- Story 9.8 complete (JWT middleware/UserAuth extractor)

## Acceptance Criteria

1. POST /api/v1/users/{userId}/devices/{deviceId}/transfer transfers device ownership
2. Requires JWT authentication via Bearer token
3. Only device owner can transfer (userId must match JWT user_id and device owner)
4. Request body requires new_owner_id (UUID of target user)
5. Returns 403 if userId doesn't match authenticated user
6. Returns 404 if device not found
7. Returns 403 if device is not owned by the requesting user
8. Returns 404 if target user not found
9. Updates owner_user_id, linked_at, clears is_primary on the device
10. Returns updated device data with new owner

## Technical Notes

- Use UserAuth extractor for JWT validation
- Leverage DeviceRepository.transfer_device_ownership() method (already implemented in Story 10.2)
- Must verify current ownership and target user existence
- Transfer clears is_primary flag (new owner decides primary)

## Implementation Tasks

- [x] Create TransferDeviceRequest DTO
- [x] Create TransferDeviceResponse DTO
- [x] Add transfer_device handler in users routes
- [x] Add route to app.rs

---

## Dev Notes

- Device data (locations, settings) remains with the device
- New owner inherits existing device data
- Transfer is immediate, no acceptance required (can be added later)
- Primary device flag is cleared on transfer

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Implemented POST /api/v1/users/:user_id/devices/:device_id/transfer endpoint
- Created TransferDeviceRequest DTO (requires new_owner_id UUID)
- Created TransferDeviceResponse DTO (includes device info, previous/new owner IDs)
- Validates device ownership before transfer
- Validates target user exists and is active
- Prevents self-transfer with validation error
- Returns 403 if device owned by another user or not linked
- Returns 404 if device or target user not found
- Uses DeviceRepository.transfer_device_ownership() which clears is_primary
- All tests pass

---

## File List

- crates/api/src/routes/users.rs (DTOs and handler)
- crates/api/src/app.rs (route registration)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Story completed |

