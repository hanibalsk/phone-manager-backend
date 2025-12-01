# Story 12.3: Update Device Settings Endpoint

**Epic**: Epic 12 - Settings Control
**Status**: In Progress
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

- [ ] Add update_device_settings method to SettingRepository
- [ ] Add update_device_setting method (single) to SettingRepository
- [ ] Create UpdateSettingsRequest and UpdateSettingsResponse DTOs
- [ ] Create UpdateSettingRequest and UpdateSettingResponse DTOs
- [ ] Add update_settings handler (bulk)
- [ ] Add update_setting handler (single)
- [ ] Implement lock bypass for admins
- [ ] Add routes to app.rs
- [ ] Add unit tests

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

