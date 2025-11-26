# Story 3.5: Group Device Listing with Last Location

**Status**: Not Started

## Story

**As a** mobile app
**I want** group device listings to include last known location
**So that** users can see where everyone is on a map

**Prerequisites**: Story 2.5, Story 3.1

## Acceptance Criteria

1. [ ] `GET /api/v1/devices?groupId=<id>` enhanced to include last location
2. [ ] Response: `{"devices": [{"deviceId": "<uuid>", "displayName": "<name>", "lastLocation": {"latitude": <float>, "longitude": <float>, "timestamp": "<iso>", "accuracy": <float>}, "lastSeenAt": "<iso>"}]}`
3. [ ] `lastLocation` is null if device has no location records
4. [ ] Uses most recent location by `captured_at` timestamp
5. [ ] Query executes in <100ms for 20 devices
6. [ ] Accuracy included to show location quality

## Technical Notes

- Use `devices_with_last_location` view created in migrations
- LATERAL join for efficient last location lookup
- Index on (device_id, captured_at DESC) enables fast lookup

## Tasks/Subtasks

- [ ] 1. Update response types
  - [ ] 1.1 Add LastLocation struct to domain models
  - [ ] 1.2 Update DeviceSummary to include optional lastLocation
- [ ] 2. Update repository
  - [ ] 2.1 Use devices_with_last_location view
  - [ ] 2.2 Map entity to domain model with location
- [ ] 3. Update handler
  - [ ] 3.1 Return enhanced response format
- [ ] 4. Write tests
  - [ ] 4.1 Test with devices with locations
  - [ ] 4.2 Test with devices without locations
- [ ] 5. Run linting and formatting checks

## Dev Notes

- DeviceWithLastLocationEntity exists in persistence layer
- View joins devices with their most recent location
- Null handling for devices without locations

## Dev Agent Record

### Debug Log

(Implementation notes will be added here)

### Completion Notes

(To be filled upon completion)

## File List

### Modified Files

(To be filled)

### New Files

(To be filled)

### Deleted Files

(None expected)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Story created from epic breakdown | Dev Agent |

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All tests pass
- [ ] Code compiles without warnings
- [ ] Code formatted with rustfmt
- [ ] Story file updated with completion notes
