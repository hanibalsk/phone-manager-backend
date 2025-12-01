# Story 10.4: Unlink Device Endpoint

**Epic**: Epic 10 - User-Device Binding
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** mobile app user
**I want** to unlink a device from my account
**So that** I can remove devices I no longer own or use

## Prerequisites

- Story 10.1 complete (device user binding columns)
- Story 10.2 complete (link device endpoint)
- Story 9.8 complete (JWT middleware/UserAuth extractor)

## Acceptance Criteria

1. DELETE /api/v1/users/{userId}/devices/{deviceId}/unlink unlinks device from user
2. Requires JWT authentication via Bearer token
3. Only allows unlinking own devices (userId must match JWT user_id)
4. Returns 403 if userId doesn't match authenticated user
5. Returns 404 if device not found
6. Returns 403 if device is not owned by the user
7. Clears owner_user_id, linked_at, is_primary on the device
8. Returns success response with unlinked status

## Technical Notes

- Use UserAuth extractor for JWT validation
- Leverage DeviceRepository.unlink_device() method (already implemented in Story 10.2)
- Must verify device ownership before unlinking

## Implementation Tasks

- [x] Add unlink_device handler in users routes
- [x] Add route to app.rs
- [x] Add unit tests (reuses existing DeviceBindingPath)

---

## Dev Notes

- Device remains in the system but is no longer linked to any user
- Device can be re-linked by same or different user later
- Unlinking clears primary device flag

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Implemented DELETE /api/v1/users/:user_id/devices/:device_id/unlink endpoint
- Created UnlinkDeviceResponse DTO
- Reuses DeviceBindingPath from link endpoint
- Verifies device ownership before unlinking (403 if not owned by user)
- Returns 404 if device not found
- Returns 403 if device owned by another user or not linked
- Uses DeviceRepository.unlink_device() which clears owner_user_id, linked_at, is_primary
- All tests pass

---

## File List

- crates/api/src/routes/users.rs (handler and DTO)
- crates/api/src/app.rs (route registration)

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Story completed |
| 2025-12-01 | Senior Developer Review notes appended |

---

## Senior Developer Review (AI)

### Reviewer
Martin Janci

### Date
2025-12-01

### Outcome
**Approve**

### Summary
Story 10.4 implementation is complete. The unlink device endpoint correctly validates ownership before unlinking and properly clears all binding fields (owner_user_id, linked_at, is_primary).

### Key Findings

**Positive Findings:**
1. ✅ **Proper ownership validation**: Verifies device is owned by requesting user
2. ✅ **Complete unlinking**: Clears owner_user_id, linked_at, and is_primary
3. ✅ **Error differentiation**: Distinguishes between "not found", "owned by another", and "not linked"

### Acceptance Criteria Coverage

| AC | Description | Status |
|----|-------------|--------|
| 1 | DELETE endpoint at correct path | ✅ Met |
| 2-3 | JWT auth + self-only restriction | ✅ Met |
| 4-6 | Returns 403 for various forbidden scenarios | ✅ Met |
| 5 | Returns 404 if device not found | ✅ Met |
| 7-8 | Clears binding fields and returns success | ✅ Met |

### Security Notes

1. ✅ Ownership verification prevents unauthorized unlink operations

### Action Items

None - implementation is approved for merge.

