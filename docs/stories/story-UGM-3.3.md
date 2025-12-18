# Story UGM-3.3: Remove Device from Group

**Status**: Complete ✅

## Story

**As a** device owner or group admin,
**I want** to remove a device from an authenticated group,
**So that** the device is no longer visible to group members.

**Epic**: UGM-3: Device-Group Management
**Prerequisites**: Story UGM-3.1: Device-Group Membership Table

## Acceptance Criteria

1. [x] Given a device owner who wants to remove their device from a group, when calling `DELETE /api/v1/groups/:groupId/devices/:deviceId`, then the device is removed from the group
2. [x] The response is 204 No Content
3. [x] The operation completes in < 100ms
4. [x] Given a group admin/owner, when removing ANY device from their group, then the device is removed successfully
5. [x] Given a regular group member (not admin/owner), when attempting to remove another user's device, then the response is 403 Forbidden
6. [x] Given a device that is not in the specified group, when attempting to remove it, then the response is 404 Not Found

## Technical Notes

- Endpoint: `DELETE /api/v1/groups/:groupId/devices/:deviceId`
- Requires JWT authentication (UserAuth extractor)
- Authorization: device owner OR group admin/owner
- Uses `device_group_memberships` table

## Tasks/Subtasks

- [x] 1. Implement remove_device_from_group handler
- [x] 2. Add authorization checks (device owner or group admin)
- [x] 3. Register route in app.rs

## File List

### Files Created

- `docs/stories/story-UGM-3.3.md` - This story file

### Files Modified

- `crates/api/src/routes/groups.rs` - Add handler for removing device from group
- `crates/api/src/app.rs` - Register new route

## Implementation Details

### Response Format

- **Success**: 204 No Content (no body)

### Error Responses

| Status | Code | Description |
|--------|------|-------------|
| 403 | Forbidden | Not device owner and not group admin |
| 404 | Not Found | Group not found or not a member |
| 404 | Not Found | Device not found |
| 404 | Not Found | Device not in group |

### Authorization

Requires JWT authentication. User must:
1. Be a member of the target group, AND
2. Either own the device OR be a group admin/owner

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass (unit tests in workspace)
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Code passes clippy
- [x] Story file updated with completion notes

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created and implemented | Dev Agent |
