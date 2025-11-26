# Story 3.1: Single Location Upload API

**Status**: Not Started

## Story

**As a** mobile app
**I want** to upload a single location point
**So that** my current location is visible to my group

**Prerequisites**: Epic 2 complete ✅

## Acceptance Criteria

1. [ ] `POST /api/v1/locations` accepts JSON: `{"deviceId": "<uuid>", "timestamp": <ms-epoch>, "latitude": <float>, "longitude": <float>, "accuracy": <float>, "altitude": <optional>, "bearing": <optional>, "speed": <optional>, "provider": <optional>, "batteryLevel": <optional>, "networkType": <optional>}`
2. [ ] Validates: latitude (-90 to 90), longitude (-180 to 180), accuracy (>= 0), bearing (0-360 if present), speed (>= 0 if present), batteryLevel (0-100 if present)
3. [ ] Returns 400 for validation errors with field-level details
4. [ ] Returns 404 if device not registered
5. [ ] Returns 200 with: `{"success": true, "processedCount": 1}`
6. [ ] Stores location with `captured_at` from timestamp, `created_at` from server time
7. [ ] Converts timestamp from milliseconds to proper DateTime
8. [ ] Updates device's last_seen_at timestamp

## Technical Notes

- Domain model in `crates/domain/src/models/location.rs`
- Repository in `crates/persistence/src/repositories/location.rs`
- Use `validator` crate for declarative validation

## Tasks/Subtasks

- [ ] 1. Create location repository
  - [ ] 1.1 Create `crates/persistence/src/repositories/location.rs`
  - [ ] 1.2 Implement `insert_location` method
  - [ ] 1.3 Export from mod.rs
- [ ] 2. Implement upload_location handler
  - [ ] 2.1 Update handler to use repository
  - [ ] 2.2 Convert millisecond timestamp to DateTime
  - [ ] 2.3 Verify device exists before insert
  - [ ] 2.4 Update device last_seen_at
- [ ] 3. Write tests
  - [ ] 3.1 Unit tests for validation
  - [ ] 3.2 Unit tests for timestamp conversion
  - [ ] 3.3 Test error responses
- [ ] 4. Run linting and formatting checks

## Dev Notes

- UploadLocationRequest already defined in domain models
- Location validation already defined with validator crate
- Need to verify device exists before allowing location upload

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
