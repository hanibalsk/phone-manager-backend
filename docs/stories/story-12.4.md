# Story 12.4: Lock/Unlock Settings Endpoints

**Epic**: Epic 12 - Settings Control
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** group admin or owner
**I want** to lock and unlock device settings
**So that** I can prevent users from modifying certain configurations

## Prerequisites

- Story 12.3 complete (Update device settings)
- RBAC middleware available

## Acceptance Criteria

1. `GET /api/v1/devices/:device_id/settings/locks` lists all locked settings
2. `POST /api/v1/devices/:device_id/settings/:key/lock` locks a setting
3. `DELETE /api/v1/devices/:device_id/settings/:key/lock` unlocks a setting
4. Lock includes optional reason and forced value
5. Only group admin or owner can manage locks
6. Returns 403 if not admin/owner
7. Returns 404 if device or setting not found
8. Returns 400 if setting is not lockable
9. Requires JWT authentication
10. Records who locked/unlocked and when

## Technical Notes

- Routes:
  - `GET /api/v1/devices/:device_id/settings/locks`
  - `POST /api/v1/devices/:device_id/settings/:key/lock`
  - `DELETE /api/v1/devices/:device_id/settings/:key/lock`
- Only settings with `is_lockable=true` in definitions can be locked
- Lock can optionally set a new value

## Implementation Tasks

- [x] Add list_setting_locks method to SettingRepository
- [x] Add lock_setting method to SettingRepository
- [x] Add unlock_setting method to SettingRepository
- [x] Create LockSettingRequest DTO
- [x] Create LockSettingResponse and UnlockSettingResponse DTOs
- [x] Create ListLocksResponse DTO
- [x] Add get_locks handler
- [x] Add lock_setting handler
- [x] Add unlock_setting handler
- [x] Add routes to app.rs
- [x] Add unit tests

## API Request Example (Lock)

```json
{
  "reason": "Company policy requires this setting",
  "value": 5,
  "notifyUser": true
}
```

## API Response Example (Lock)

```json
{
  "key": "tracking_interval_minutes",
  "isLocked": true,
  "value": 5,
  "lockedBy": "user_01ADMIN456",
  "lockedAt": "2025-12-01T10:35:00Z",
  "reason": "Company policy requires this setting"
}
```

---

## Dev Notes

- Non-lockable settings (per definition) return 400 on lock attempt
- Locking with value forces the setting to that value
- Push notification support deferred to Story 12.8

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Added get_setting_locks handler to list all locked settings for a device
- Added lock_setting handler with is_lockable validation
- Added unlock_setting handler with admin-only authorization
- Lock includes optional reason and forced value
- Non-lockable settings return validation error (400)
- Response includes locker info with display name
- All routes added with proper JWT auth
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

