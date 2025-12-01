# Story 3.1: Single Location Upload API

**Status**: Complete ✅

## Story

**As a** mobile app
**I want** to upload a single location point
**So that** my current location is visible to my group

**Prerequisites**: Epic 2 complete ✅

## Acceptance Criteria

1. [x] `POST /api/v1/locations` accepts JSON: `{"device_id": "<uuid>", "timestamp": <ms-epoch>, "latitude": <float>, "longitude": <float>, "accuracy": <float>, "altitude": <optional>, "bearing": <optional>, "speed": <optional>, "provider": <optional>, "batteryLevel": <optional>, "networkType": <optional>}`
2. [x] Validates: latitude (-90 to 90), longitude (-180 to 180), accuracy (>= 0), bearing (0-360 if present), speed (>= 0 if present), batteryLevel (0-100 if present)
3. [x] Returns 400 for validation errors with field-level details
4. [x] Returns 404 if device not registered
5. [x] Returns 200 with: `{"success": true, "processedCount": 1}`
6. [x] Stores location with `captured_at` from timestamp, `created_at` from server time
7. [x] Converts timestamp from milliseconds to proper DateTime
8. [x] Updates device's last_seen_at timestamp

## Technical Notes

- Domain model in `crates/domain/src/models/location.rs`
- Repository in `crates/persistence/src/repositories/location.rs`
- Use `validator` crate for declarative validation

## Tasks/Subtasks

- [x] 1. Create location repository
  - [x] 1.1 Create `crates/persistence/src/repositories/location.rs`
  - [x] 1.2 Implement `insert_location` method
  - [x] 1.3 Export from mod.rs
- [x] 2. Implement upload_location handler
  - [x] 2.1 Update handler to use repository
  - [x] 2.2 Convert millisecond timestamp to DateTime
  - [x] 2.3 Verify device exists before insert
  - [x] 2.4 Update device last_seen_at
- [x] 3. Write tests
  - [x] 3.1 Unit tests for validation
  - [x] 3.2 Unit tests for timestamp conversion
  - [x] 3.3 Test error responses
- [x] 4. Run linting and formatting checks

## Dev Notes

- UploadLocationRequest already defined in domain models
- Location validation already defined with validator crate
- Need to verify device exists before allowing location upload

## Dev Agent Record

### Debug Log

- Implemented location repository with insert_location method
- Handler validates device existence before insert
- Millisecond timestamp converted to DateTime<Utc>
- Device last_seen_at updated after successful upload

### Completion Notes

Single location upload fully functional with validation, device verification, and timestamp handling.

## File List

### Modified Files

- `crates/api/src/routes/locations.rs` - upload_location handler
- `crates/persistence/src/repositories/mod.rs` - export location repository

### New Files

- `crates/persistence/src/repositories/location.rs` - location repository

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

---

## Senior Developer Review (AI)

### Reviewer: Martin Janci
### Date: 2025-11-26
### Outcome: ✅ Approve

### Summary
Single location upload API properly implemented with comprehensive validation, device verification, and timestamp conversion from milliseconds.

### Key Findings
- **[Info]** Validator crate provides declarative validation
- **[Info]** Device existence check prevents orphaned locations
- **[Info]** Millisecond to DateTime conversion handles client timestamps

### Acceptance Criteria Coverage
| AC | Status | Evidence |
|----|--------|----------|
| AC1 - POST endpoint accepts JSON | ✅ | UploadLocationRequest struct |
| AC2 - Validation rules | ✅ | Validator annotations |
| AC3 - 400 for validation errors | ✅ | ApiError::Validation |
| AC4 - 404 for unregistered device | ✅ | Device existence check |
| AC5 - 200 with processedCount | ✅ | UploadResponse struct |
| AC6 - captured_at from timestamp | ✅ | Timestamp conversion |
| AC7 - Millisecond conversion | ✅ | DateTime::from_timestamp_millis |
| AC8 - Updates last_seen_at | ✅ | Device update call |

### Test Coverage and Gaps
- Validation tests comprehensive
- Integration tests cover happy path and errors
- No gaps identified

### Architectural Alignment
- ✅ Follows layered architecture pattern
- ✅ Repository pattern for data access

### Security Notes
- Device verification prevents unauthorized location uploads

### Action Items
None - story approved for completion.

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Senior Developer Review notes appended | AI Reviewer |
