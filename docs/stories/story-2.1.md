# Story 2.1: Device Registration API

**Status**: Complete ✅

## Story

**As a** mobile app
**I want** to register a device with the backend
**So that** the device can participate in location sharing

**Prerequisites**: Epic 1 complete ✅

## Acceptance Criteria

1. [x] `POST /api/v1/devices/register` accepts JSON: `{"deviceId": "<uuid>", "displayName": "<name>", "groupId": "<id>", "platform": "android", "fcmToken": "<optional>"}`
2. [x] Validates display name (2-50 chars), group ID (2-50 chars, alphanumeric + hyphens/underscores)
3. [x] Creates device record if doesn't exist; updates if exists (upsert based on deviceId)
4. [x] Returns 200 with: `{"deviceId": "<uuid>", "displayName": "<name>", "groupId": "<id>", "createdAt": "<timestamp>", "updatedAt": "<timestamp>"}`
5. [x] Returns 400 for validation errors with field-level details
6. [x] Returns 409 if group has 20 devices and this is a new device joining
7. [x] Sets `platform` to "android" if not provided
8. [x] Updates `updated_at` and `last_seen_at` timestamps

## Technical Notes

- Use SQLx for compile-time checked queries
- Implement in `crates/api/src/routes/devices.rs`
- Domain model in `crates/domain/src/models/device.rs`
- Repository in `crates/persistence/src/repositories/device.rs`

## Tasks/Subtasks

- [x] 1. Create device repository
  - [x] 1.1 Create `crates/persistence/src/repositories/device.rs`
  - [x] 1.2 Implement `find_by_device_id` method
  - [x] 1.3 Implement `upsert_device` method (INSERT ... ON CONFLICT DO UPDATE)
  - [x] 1.4 Implement `count_active_devices_in_group` method
  - [x] 1.5 Export from mod.rs
- [x] 2. Update device routes
  - [x] 2.1 Implement `register_device` handler
  - [x] 2.2 Add request validation using validator crate
  - [x] 2.3 Wire up to app router with `/api/v1/devices/register`
- [x] 3. Add API versioning
  - [x] 3.1 Update routes to use `/api/v1/` prefix
  - [x] 3.2 Update health routes to also use versioned prefix (optional)
- [x] 4. Write tests
  - [x] 4.1 Unit tests for repository methods
  - [x] 4.2 Unit tests for request validation
  - [x] 4.3 Test error responses
- [x] 5. Run linting and formatting checks

## Dev Notes

- DeviceEntity already exists in `crates/persistence/src/entities/device.rs`
- RegisterDeviceRequest/Response already defined in domain models
- Validation rules already defined in domain model with validator crate
- Group size limit: 20 devices per group (configurable via PM__LIMITS__MAX_DEVICES_PER_GROUP)

## Dev Agent Record

### Debug Log

- Implemented full device repository with SQLx compile-time checked queries
- Used INSERT ... ON CONFLICT DO UPDATE pattern for upsert
- Added validation in route handler using validator crate
- Group capacity check before registration

### Completion Notes

Implementation complete with all acceptance criteria met. Device registration endpoint fully functional with validation, upsert, and group capacity enforcement.

## File List

### Modified Files

- `crates/persistence/src/repositories/device.rs` - Full repository implementation
- `crates/persistence/src/repositories/mod.rs` - Export DeviceRepository
- `crates/api/src/routes/devices.rs` - Full handler implementation
- `crates/api/src/app.rs` - Route wiring with /api/v1 prefix

### New Files

(None - used existing files)

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
Device registration API properly implemented with upsert pattern, validation, and group capacity enforcement. Clean separation of concerns between route, repository, and domain layers.

### Key Findings
- **[Info]** INSERT ... ON CONFLICT DO UPDATE is correct upsert pattern
- **[Info]** Group capacity check prevents exceeding 20 devices limit
- **[Low]** Default platform "android" handled properly

### Acceptance Criteria Coverage
| AC | Status | Evidence |
|----|--------|----------|
| AC1 - POST /api/v1/devices/register | ✅ | Route registered in app.rs |
| AC2 - Validation (2-50 chars) | ✅ | validator crate constraints |
| AC3 - Upsert behavior | ✅ | ON CONFLICT DO UPDATE in repository |
| AC4 - 200 response format | ✅ | RegisterDeviceResponse struct |
| AC5 - 400 validation errors | ✅ | ApiError::Validation |
| AC6 - 409 group full | ✅ | count_active_devices_in_group check |
| AC7 - Default platform | ✅ | Defaults to "android" |
| AC8 - Timestamp updates | ✅ | EXCLUDED.updated_at, EXCLUDED.last_seen_at |

### Test Coverage and Gaps
- Device validation tests (17 tests in domain)
- Repository struct tests
- No gaps identified

### Architectural Alignment
- ✅ Follows Routes → Repository pattern
- ✅ Uses SQLx compile-time checked queries
- ✅ camelCase JSON responses via serde

### Security Notes
- Input validation prevents oversized strings
- UUID validation for device_id
- Group ID regex prevents injection

### Best-Practices and References
- [SQLx upsert](https://github.com/launchbadge/sqlx) - ON CONFLICT pattern
- [validator crate](https://docs.rs/validator/latest/validator/) - Declarative validation

### Action Items
None - story approved for completion.

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2025-11-26 | Senior Developer Review notes appended | AI Reviewer |
