# Story 12.5: Bulk Lock Update Endpoint

**Epic**: Epic 12 - Settings Control
**Status**: In Progress
**Created**: 2025-12-01

---

## User Story

**As a** group admin or owner
**I want** to update multiple setting locks at once
**So that** I can efficiently manage device restrictions

## Prerequisites

- Story 12.4 complete (Lock/unlock settings)
- Individual lock/unlock endpoints operational

## Acceptance Criteria

1. `PUT /api/v1/devices/:device_id/settings/locks` updates multiple locks
2. Request contains map of setting keys to lock state (boolean)
3. Optional reason applies to all changes
4. Optional notify_user triggers push notification
5. Only group admin or owner can access
6. Returns list of updated locks
7. Skips non-lockable settings with warning
8. Returns 403 if not admin/owner
9. Returns 404 if device not found
10. Requires JWT authentication

## Technical Notes

- Route: `PUT /api/v1/devices/:device_id/settings/locks`
- Atomic transaction for all lock changes
- Response includes what was updated vs skipped

## Implementation Tasks

- [ ] Add bulk_update_locks method to SettingRepository
- [ ] Create BulkUpdateLocksRequest DTO
- [ ] Create BulkUpdateLocksResponse DTO
- [ ] Add bulk_update_locks handler
- [ ] Add route to app.rs
- [ ] Add unit tests

## API Request Example

```json
{
  "locks": {
    "tracking_enabled": true,
    "tracking_interval_minutes": true,
    "secret_mode_enabled": true,
    "movement_detection_enabled": false
  },
  "reason": "Updated security policy",
  "notifyUser": true
}
```

## API Response Example

```json
{
  "updated": [
    {
      "key": "tracking_enabled",
      "isLocked": true,
      "lockedAt": "2025-12-01T10:35:00Z"
    },
    {
      "key": "movement_detection_enabled",
      "isLocked": false,
      "unlockedAt": "2025-12-01T10:35:00Z"
    }
  ],
  "skipped": [
    {
      "key": "battery_optimization_enabled",
      "reason": "Setting is not lockable"
    }
  ],
  "notificationSent": true
}
```

---

## Dev Notes

- Efficient for applying policy-like changes
- Non-lockable settings reported but don't fail request
- Push notification support deferred to Story 12.8

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

