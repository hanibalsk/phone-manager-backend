# Story UGM-3.2: Add Device to Group

**Status**: Complete ✅

## Story

**As a** group member,
**I want** to add my device to an authenticated group,
**So that** other group members can see my location.

**Epic**: UGM-3: Device-Group Management
**Prerequisites**: Story UGM-3.1: Device-Group Membership Table

## Acceptance Criteria

1. [x] Given an authenticated user who is a member of a group, when the user calls `POST /api/v1/groups/:groupId/devices` with their device_id, then the device is added to the group
2. [x] The response includes `group_id`, `device_id`, `added_at`
3. [x] The operation completes in < 100ms
4. [x] Given a user who is NOT a member of the group, when attempting to add a device, then the response is 403 Forbidden
5. [x] Given a user attempting to add a device they don't own, when the request is made, then the response is 403 Forbidden
6. [x] Given a device that is already in the group, when attempting to add it again, then the response is 409 Conflict
7. [x] Given an invalid group_id or device_id, when the request is made, then the response is 404 Not Found

## Technical Notes

- Endpoint: `POST /api/v1/groups/:groupId/devices`
- Requires JWT authentication (UserAuth extractor)
- Uses `device_group_memberships` table for multi-group support
- Authorization checks: user must be group member AND device owner

## Tasks/Subtasks

- [x] 1. Add AddDeviceToGroupRequest struct
- [x] 2. Add AddDeviceToGroupResponse struct
- [x] 3. Implement add_device_to_group handler
- [x] 4. Register route in app.rs

## File List

### Files Created

- `docs/stories/story-UGM-3.2.md` - This story file

### Files Modified

- `crates/api/src/routes/groups.rs` - Add handler for adding device to group
- `crates/api/src/app.rs` - Register new route

## Implementation Details

### Request Body

```json
{
  "device_id": "uuid"
}
```

### Response Format (201 Created)

```json
{
  "group_id": "uuid",
  "device_id": "uuid",
  "added_at": "2025-12-18T10:30:00Z"
}
```

### Error Responses

| Status | Code | Description |
|--------|------|-------------|
| 403 | Forbidden | Not a group member |
| 403 | Forbidden | Not device owner |
| 404 | Not Found | Device not found |
| 409 | Conflict | Device already in group |

### Authorization

Requires JWT authentication. User must:
1. Be a member of the target group
2. Own the device being added

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
