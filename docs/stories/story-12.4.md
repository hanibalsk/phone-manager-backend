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
Story 12.4 implements lock/unlock endpoints with proper admin-only authorization, lockability validation, and optional value forcing on lock. The implementation correctly records who locked/unlocked settings and when, providing full audit trail.

### Key Findings

**Positive Observations**:
- `is_lockable` validation from setting definitions prevents locking non-lockable settings
- Lock includes optional reason and forced value
- Response includes locker info with display name for transparency
- Unlock requires setting to be locked (returns 404 if not)
- Repository methods properly manage lock metadata (locked_by, locked_at)

**Severity: None**
- All requirements implemented correctly

### Acceptance Criteria Coverage

| AC# | Criterion | Status | Evidence |
|-----|-----------|--------|----------|
| 1 | GET /locks lists all locked settings | ✅ Pass | `get_setting_locks` handler line 456 |
| 2 | POST /lock locks a setting | ✅ Pass | `lock_setting` handler line 530 |
| 3 | DELETE /lock unlocks a setting | ✅ Pass | `unlock_setting` handler line 639 |
| 4 | Lock includes optional reason and forced value | ✅ Pass | LockSettingRequest with reason, value fields |
| 5 | Only group admin/owner can manage locks | ✅ Pass | `check_is_admin()` call at lines 547, 655 |
| 6 | Returns 403 if not admin/owner | ✅ Pass | Lines 549-553, 657-661: ApiError::Forbidden |
| 7 | Returns 404 if device/setting not found | ✅ Pass | Lines 544, 559, 652, 667 |
| 8 | Returns 400 if setting not lockable | ✅ Pass | Lines 562-567: ApiError::Validation |
| 9 | Requires JWT authentication | ✅ Pass | UserAuth extractor in all handlers |
| 10 | Records who locked/unlocked and when | ✅ Pass | locked_by, locked_at in responses |

### Test Coverage and Gaps
- ✅ Repository tests cover lock/unlock operations
- ✅ Entity serialization tested
- **No gap identified**

### Architectural Alignment
✅ Consistent patterns:
- Admin check reused across handlers
- Proper error mapping to API errors
- Logging for observability

### Security Notes
- ✅ Admin-only access enforced
- ✅ Lock value validated against data type when provided
- ✅ Audit trail with locked_by and locked_at fields

### Action Items
None - Story approved as implemented.

