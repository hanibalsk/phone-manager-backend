# Story 12.7: Settings Sync Endpoint

**Epic**: Epic 12 - Settings Control
**Status**: In Progress
**Created**: 2025-12-01

---

## User Story

**As a** device
**I want** to sync my settings from the server
**So that** I can ensure my configuration matches server state

## Prerequisites

- Story 12.2 complete (Get device settings)
- Device token or JWT authentication

## Acceptance Criteria

1. `POST /api/v1/devices/:device_id/settings/sync` triggers settings sync
2. Returns complete settings state with all values and lock status
3. Includes list of changes since last sync
4. Updates `last_synced_at` timestamp on device settings
5. Requires device token or user JWT authentication
6. Only device owner can trigger sync
7. Returns 404 if device not found
8. Returns 403 if not authorized
9. Response includes settings that changed since last sync

## Technical Notes

- Route: `POST /api/v1/devices/:device_id/settings/sync`
- Track `last_synced_at` per device
- Compare current settings with previous sync state
- Can be called by device on startup or periodically

## Implementation Tasks

- [ ] Add last_synced_at column to device_settings tracking (or separate table)
- [ ] Create sync_settings method in SettingRepository
- [ ] Create SyncSettingsResponse DTO
- [ ] Add sync_settings handler
- [ ] Add route to app.rs
- [ ] Add unit tests

## API Response Example

```json
{
  "syncedAt": "2025-12-01T10:40:00Z",
  "settings": {
    "tracking_enabled": {
      "value": true,
      "isLocked": false
    },
    "tracking_interval_minutes": {
      "value": 5,
      "isLocked": true
    }
  },
  "changesApplied": [
    {
      "key": "tracking_interval_minutes",
      "oldValue": 10,
      "newValue": 5,
      "reason": "Admin changed value"
    }
  ]
}
```

---

## Dev Notes

- Sync is idempotent - can be called multiple times safely
- Changes list shows what admin modified since last sync
- Device should call sync on app launch and when receiving push notification

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

