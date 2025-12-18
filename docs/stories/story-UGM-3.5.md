# Story UGM-3.5: View Device's Group Memberships

**Status**: Complete ✅

## Story

**As a** device owner,
**I want** to see which groups my device belongs to,
**So that** I can manage my device's visibility across groups.

**Epic**: UGM-3: Device-Group Management
**Prerequisites**: Story UGM-3.1: Device-Group Membership Table

## Acceptance Criteria

1. [x] Given an authenticated user viewing their device details, when calling `GET /api/v1/devices/:deviceId/groups`, then the response includes a list of all groups the device belongs to
2. [x] Each group includes: `group_id`, `name`, `slug`, `role` (user's role in group), `added_at`
3. [x] Given a user viewing a device they don't own, when calling `GET /api/v1/devices/:deviceId/groups`, then the response is 403 Forbidden
4. [x] Given a device with no group memberships, when calling `GET /api/v1/devices/:deviceId/groups`, then the response includes an empty list

## Technical Notes

- Endpoint: `GET /api/v1/devices/:deviceId/groups`
- Requires JWT authentication (UserAuth extractor)
- Uses `device_group_memberships` table joined with `groups` and `group_memberships`
- Authorization: only device owner can view their device's groups

## Tasks/Subtasks

- [x] 1. Add DeviceGroupMembershipInfo struct
- [x] 2. Add ListDeviceGroupsResponse struct
- [x] 3. Implement list_device_groups handler
- [x] 4. Register route in app.rs (user_routes section)

## File List

### Files Created

- `docs/stories/story-UGM-3.5.md` - This story file

### Files Modified

- `crates/api/src/routes/groups.rs` - Add handler for listing device's groups
- `crates/api/src/app.rs` - Register new route in user_routes

## Implementation Details

### Response Format

```json
{
  "groups": [
    {
      "group_id": "uuid",
      "name": "Chen Family",
      "slug": "chen-family",
      "role": "member",
      "added_at": "2025-12-18T10:30:00Z"
    }
  ]
}
```

### Error Responses

| Status | Code | Description |
|--------|------|-------------|
| 403 | Forbidden | Not device owner |
| 404 | Not Found | Device not found |

### Authorization

Requires JWT authentication. User must own the device to view its group memberships.

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
