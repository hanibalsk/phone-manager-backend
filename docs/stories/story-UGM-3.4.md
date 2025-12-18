# Story UGM-3.4: List Group Devices

**Status**: Complete ✅

## Story

**As a** group member,
**I want** to see all devices in my group with their details and locations,
**So that** I can track my family members.

**Epic**: UGM-3: Device-Group Management
**Prerequisites**: Story UGM-3.1: Device-Group Membership Table

## Acceptance Criteria

1. [x] Given an authenticated user who is a member of a group, when calling `GET /api/v1/groups/:groupId/devices/members`, then the response includes a paginated list of all devices in the group
2. [x] Each device includes: `device_id`, `display_name`, `owner_user_id`, `owner_display_name`, `added_at`, `last_seen_at`
3. [x] The response completes in < 200ms
4. [x] Given the query parameter `include_location=true`, when listing devices, then each device includes `last_location` with latitude, longitude, accuracy, timestamp
5. [x] Given pagination parameters `page=2&per_page=10`, when listing devices, then the response includes the correct page of results
6. [x] Pagination metadata shows total, page, per_page, total_pages
7. [x] Given a user who is NOT a member of the group, when attempting to list devices, then the response is 403 Forbidden

## Technical Notes

- Endpoint: `GET /api/v1/groups/:groupId/devices/members`
- Requires JWT authentication (UserAuth extractor)
- Uses `device_group_memberships` table for multi-group support
- Query parameters: `include_location`, `page`, `per_page`
- Default pagination: page=1, per_page=20
- Max per_page: 100

## Tasks/Subtasks

- [x] 1. Add ListGroupDevicesQuery struct for query parameters
- [x] 2. Add GroupDeviceInfo struct for device details
- [x] 3. Add DeviceLocationInfo struct for location data
- [x] 4. Add ListGroupDevicesResponse struct
- [x] 5. Implement list_group_devices handler
- [x] 6. Register route in app.rs

## File List

### Files Created

- `docs/stories/story-UGM-3.4.md` - This story file

### Files Modified

- `crates/api/src/routes/groups.rs` - Add handler for listing group devices
- `crates/api/src/app.rs` - Register new route

## Implementation Details

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `include_location` | bool | false | Include last location for each device |
| `page` | i64 | 1 | Page number (1-based) |
| `per_page` | i64 | 20 | Items per page (1-100) |

### Response Format

```json
{
  "data": [
    {
      "device_id": "uuid",
      "display_name": "John's iPhone",
      "owner_user_id": "uuid",
      "owner_display_name": "John",
      "added_at": "2025-12-18T10:30:00Z",
      "last_seen_at": "2025-12-18T10:30:00Z",
      "last_location": {
        "latitude": 37.7749,
        "longitude": -122.4194,
        "accuracy": 10.0,
        "timestamp": "2025-12-18T10:30:00Z"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

### Authorization

Requires JWT authentication. User must be a member of the target group.

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
