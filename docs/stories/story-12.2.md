# Story 12.2: Get Device Settings Endpoint

**Epic**: Epic 12 - Settings Control
**Status**: In Progress
**Created**: 2025-12-01

---

## User Story

**As a** device owner or admin
**I want** to retrieve all settings for a device
**So that** I can view current configuration and lock states

## Prerequisites

- Story 12.1 complete (Device settings schema)
- JWT authentication operational
- RBAC middleware available

## Acceptance Criteria

1. `GET /api/v1/devices/:device_id/settings` returns all device settings
2. Response includes setting values, lock status, and metadata
3. Optional `include_definitions=true` query param includes setting definitions
4. Requires JWT authentication
5. Device owner, group admin, or org admin can access
6. Returns 404 if device not found
7. Returns 403 if not authorized
8. Settings not explicitly set use default values from definitions

## Technical Notes

- Route: `GET /api/v1/devices/:device_id/settings`
- Requires `UserAuth` extractor
- Check device ownership or admin role in device's group
- Merge explicit settings with defaults from definitions

## Implementation Tasks

- [ ] Create SettingRepository with get_device_settings method
- [ ] Create GetSettingsResponse DTO
- [ ] Add get_device_settings handler in device settings routes
- [ ] Add authorization checks (owner, group admin, org admin)
- [ ] Add route to app.rs
- [ ] Add unit tests

## API Response Example

```json
{
  "deviceId": "...",
  "settings": {
    "tracking_enabled": {
      "value": true,
      "isLocked": false,
      "updatedAt": "2025-12-01T10:30:00Z"
    },
    "tracking_interval_minutes": {
      "value": 5,
      "isLocked": true,
      "lockedBy": "user_id",
      "lockedAt": "2025-12-01T09:00:00Z",
      "lockReason": "Company policy"
    }
  },
  "lastSyncedAt": "2025-12-01T10:30:00Z"
}
```

---

## Dev Notes

- Settings not in device_settings table use defaults from definitions
- Locked settings include locker info for transparency

---

## Dev Agent Record

### Debug Log


### Completion Notes


---

## File List

- crates/persistence/src/repositories/setting.rs
- crates/persistence/src/repositories/mod.rs
- crates/domain/src/models/setting.rs
- crates/api/src/routes/device_settings.rs
- crates/api/src/routes/mod.rs
- crates/api/src/app.rs

---

## Change Log

| Date | Change |
|------|--------|
| 2025-12-01 | Story created |

