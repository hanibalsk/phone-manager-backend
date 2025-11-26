# Story 2.5: Group Device Listing API

**Status**: Ready for Review

## Story

**As a** mobile app
**I want** to retrieve all active devices in a group
**So that** users can see who is sharing their location

**Prerequisites**: Story 2.1 ✅

## Acceptance Criteria

1. [x] `GET /api/v1/devices?groupId=<id>` returns JSON: `{"devices": [{"deviceId": "<uuid>", "displayName": "<name>", "lastSeenAt": "<timestamp>"}]}`
2. [x] Only returns active devices (active=true)
3. [x] Sorted by `display_name` ascending
4. [x] Returns empty array if group doesn't exist or has no active devices
5. [x] Returns 400 if `groupId` query parameter missing
6. [x] Query executes in <100ms for groups with 20 devices

## Technical Notes

- Simple query: `SELECT device_id, display_name, last_seen_at FROM devices WHERE group_id=? AND active=true ORDER BY display_name`
- Will be enhanced in Epic 3 to include last location

## Tasks/Subtasks

- [x] 1. Add list devices method to repository
  - [x] 1.1 Implement `find_active_devices_by_group` method
  - [x] 1.2 Return DeviceSummary list
- [x] 2. Update get_group_devices handler
  - [x] 2.1 Wire up to repository
  - [x] 2.2 Return proper response format
  - [x] 2.3 Update route to /api/v1/devices
- [x] 3. Write tests
  - [x] 3.1 Test returns only active devices
  - [x] 3.2 Test sorted by display_name
  - [x] 3.3 Test empty array for non-existent group
  - [x] 3.4 Test 400 for missing groupId
- [x] 4. Run linting and formatting checks

## Dev Notes

- DeviceSummary already defined in domain models
- GetDevicesQuery and GetDevicesResponse already exist in routes/devices.rs

## Dev Agent Record

### Debug Log

- Implemented `find_active_devices_by_group` with ORDER BY display_name ASC
- Handler maps DeviceEntity to DeviceSummary
- Returns 400 Validation error if groupId missing

### Completion Notes

Group device listing fully functional. Returns sorted active devices with last_seen_at timestamps.

## File List

### Modified Files

- `crates/persistence/src/repositories/device.rs` - find_active_devices_by_group method
- `crates/api/src/routes/devices.rs` - get_group_devices handler

### New Files

(None)

### Deleted Files

(None)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |
| 2025-11-26 | Implementation complete | Dev Agent |

## Definition of Done

- [x] All acceptance criteria met
- [x] All tests pass
- [x] Code compiles without warnings
- [x] Code formatted with rustfmt
- [x] Story file updated with completion notes
