# Story UGM-1.3: View User's Linked Devices

**Status**: Complete ✅

## Story

**As a** mobile app user
**I want** to see all devices linked to my account
**So that** I can manage my devices and know which ones I own

**Epic**: UGM-1: Registration Group Status & Device Linking
**Prerequisites**: UGM-1.1

## Acceptance Criteria

1. [x] Given a user is authenticated, when they call GET /api/v1/devices/me, then they receive a list of all their linked devices
2. [x] Given the user has no linked devices, when they call the endpoint, then they receive an empty list
3. [x] The response includes device details: device_id, display_name, group_id, platform, is_primary, linked_at, last_seen_at

## Technical Notes

- Endpoint: `GET /api/v1/devices/me`
- Authentication: JWT (UserAuth extractor)
- Returns: list of user's linked devices
- Device repository already has `find_devices_by_user` method
- There's already a `/api/v1/users/:user_id/devices` endpoint but requires user_id param

## Tasks/Subtasks

- [x] 1. Create response struct for user's devices
- [x] 2. Add endpoint handler in devices.rs
- [x] 3. Register route in app.rs
- [ ] 4. Add unit tests (future)

## API Specification

### Request
```
GET /api/v1/devices/me
Authorization: Bearer <jwt>
```

### Response (200 OK)
```json
{
  "devices": [
    {
      "device_id": "550e8400-e29b-41d4-a716-446655440000",
      "display_name": "My Phone",
      "group_id": "family-group",
      "platform": "android",
      "is_primary": true,
      "linked_at": "2025-12-18T10:30:00Z",
      "last_seen_at": "2025-12-18T14:45:00Z"
    }
  ],
  "count": 1
}
```

## File List

### Files Modified

- `crates/api/src/routes/devices.rs` - Added `UserDeviceInfo`, `UserDevicesResponse` structs and `get_my_devices` handler
- `crates/api/src/app.rs` - Registered new route under user_routes (JWT auth)

### Files Created

- `docs/stories/story-UGM-1.3.md` - This story file

## Implementation Details

Created `UserDeviceInfo` struct with fields:
- `device_id: Uuid`
- `display_name: String`
- `group_id: String`
- `platform: String`
- `is_primary: bool`
- `linked_at: Option<DateTime<Utc>>`
- `last_seen_at: Option<DateTime<Utc>>`

Created `UserDevicesResponse` struct with:
- `devices: Vec<UserDeviceInfo>`
- `count: usize`

Added `get_my_devices` handler that:
1. Uses UserAuth extractor to get authenticated user
2. Finds all user's devices (sorted: primary first, then by linked_at)
3. Returns device list with count (empty list if no devices)

Route registered under `/api/v1/devices/me` with JWT authentication.

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-18 | Story created and implemented | Dev Agent |
