# Story 12.3: Update Device Settings Endpoint

**Epic**: Epic 12 - Settings Control
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** device owner or admin
**I want** to update device settings
**So that** I can configure device behavior remotely

## Prerequisites

- Story 12.2 complete (Get device settings endpoint)
- Device settings schema operational

## Acceptance Criteria

1. `PUT /api/v1/devices/:device_id/settings` updates multiple settings
2. `PUT /api/v1/devices/:device_id/settings/:key` updates single setting
3. Locked settings are skipped for regular users (silent fail)
4. Admins can override locked settings with `force=true` query param
5. Returns which settings were updated vs locked
6. Validates values against setting definition data types
7. Returns 400 for invalid values
8. Returns 403 if not authorized
9. Returns 404 if device not found
10. Requires JWT authentication

## Technical Notes

- Routes:
  - `PUT /api/v1/devices/:device_id/settings`
  - `PUT /api/v1/devices/:device_id/settings/:key`
- Use UserAuth extractor
- Authorization: owner (unlocked only), admin (all or force locked)
- Value validation based on `data_type` in setting_definitions

## Implementation Tasks

- [x] Add update_device_settings method to SettingRepository
- [x] Add update_device_setting method (single) to SettingRepository
- [x] Create UpdateSettingsRequest and UpdateSettingsResponse DTOs
- [x] Create UpdateSettingRequest and UpdateSettingResponse DTOs
- [x] Add update_settings handler (bulk)
- [x] Add update_setting handler (single)
- [x] Implement lock bypass for admins
- [x] Add routes to app.rs
- [x] Add unit tests

## API Request Example (Bulk)

```json
{
  "settings": {
    "tracking_enabled": true,
    "tracking_interval_minutes": 10
  }
}
```

## API Response Example

```json
{
  "updated": ["tracking_enabled"],
  "locked": ["tracking_interval_minutes"],
  "settings": {
    "tracking_enabled": {
      "value": true,
      "isLocked": false,
      "updatedAt": "2025-12-01T10:35:00Z"
    },
    "tracking_interval_minutes": {
      "value": 5,
      "isLocked": true,
      "error": "Setting is locked by admin"
    }
  }
}
```

---

## Dev Notes

- Locked settings silently skipped (no error) for regular users
- Response clearly indicates which settings couldn't be updated
- Admins can use force=true to override locks temporarily

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Added update_device_settings bulk handler for updating multiple settings at once
- Added update_device_setting single handler for updating individual settings
- Implemented value type validation based on setting definition data types
- Admin force override with `force=true` query parameter to bypass locks
- Locked settings silently skipped for non-admin users (no error thrown)
- Response includes which settings were updated, locked, and invalid
- Added comprehensive unit tests for value type validation
- All tests pass (73+ unit tests across workspace)

---

## File List

- crates/persistence/src/repositories/setting.rs
- crates/domain/src/models/setting.rs
- crates/api/src/routes/device_settings.rs
- crates/api/src/app.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |
| 2025-12-01 | Senior Developer Review (AI) notes appended |

---

## Senior Developer Review (AI)

### Reviewer
Martin Janci

### Date
2025-12-01

### Outcome
**Approve**

### Summary
Story 12.3 implements both bulk and single setting update endpoints with proper value type validation, lock bypass for admins, and detailed response showing which settings were updated/locked/invalid. The implementation correctly enforces lock constraints while providing admin override capability.

### Key Findings

**Positive Observations**:
- `validate_value_type()` function properly validates JSONB values against setting definition data types
- Admin force override via `?force=true` query parameter
- Locked settings silently skipped for non-admins (returns in `locked` array, not error)
- Response clearly indicates updated/locked/invalid settings
- Comprehensive unit tests for value type validation

**Severity: None**
- Implementation matches all acceptance criteria

### Acceptance Criteria Coverage

| AC# | Criterion | Status | Evidence |
|-----|-----------|--------|----------|
| 1 | PUT bulk updates multiple settings | ✅ Pass | `update_device_settings` handler |
| 2 | PUT single updates one setting | ✅ Pass | `update_device_setting` handler |
| 3 | Locked settings skipped (silent fail) | ✅ Pass | Lines 277-294: Added to `locked` array |
| 4 | Admins can override with `force=true` | ✅ Pass | Lines 214-216, 299-307 |
| 5 | Returns updated vs locked lists | ✅ Pass | UpdateSettingsResponse with updated/locked arrays |
| 6 | Validates values against data types | ✅ Pass | `validate_value_type()` at line 880 |
| 7 | Returns 400 for invalid values | ✅ Pass | Line 393-397: ApiError::Validation |
| 8 | Returns 403 if not authorized | ✅ Pass | Line 208: ApiError::Forbidden |
| 9 | Returns 404 if device not found | ✅ Pass | Line 196: ApiError::NotFound |
| 10 | Requires JWT authentication | ✅ Pass | UserAuth extractor |

### Test Coverage and Gaps
- ✅ Comprehensive unit tests for `validate_value_type()` (lines 1529-1619)
- ✅ Tests cover boolean, integer, string, float, and JSON types
- ✅ Tests verify negative cases (wrong types rejected)

### Architectural Alignment
✅ Consistent with project patterns:
- Uses same authorization helpers as Story 12.2
- Repository methods `upsert_setting` and `upsert_setting_force` properly separated
- Response DTOs clearly structured

### Security Notes
- ✅ Value type validation prevents type confusion attacks
- ✅ Lock enforcement prevents unauthorized modifications
- ✅ Admin force requires is_admin check first

### Action Items
None - Story approved as implemented.

