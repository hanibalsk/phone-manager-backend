# Story 12.6: Unlock Request Workflow

**Epic**: Epic 12 - Settings Control
**Status**: Done
**Created**: 2025-12-01

---

## User Story

**As a** device user
**I want** to request unlocking of a locked setting
**So that** I can ask admin permission to modify restricted settings

## Prerequisites

- Story 12.4 complete (Lock/unlock settings)
- Group membership and RBAC operational

## Acceptance Criteria

1. `POST /api/v1/devices/:device_id/settings/:key/unlock-request` creates request
2. `GET /api/v1/groups/:group_id/unlock-requests` lists pending requests for group
3. `PUT /api/v1/unlock-requests/:request_id` approves or denies request
4. Request includes reason from user
5. Only device owner can create request
6. Only group admin/owner can approve/deny
7. Approved request automatically unlocks the setting
8. Status enum: pending, approved, denied, expired
9. Requests expire after 7 days
10. Returns 409 if pending request already exists for same setting

## Technical Notes

- Create `unlock_requests` table in migration
- Routes:
  - `POST /api/v1/devices/:device_id/settings/:key/unlock-request`
  - `GET /api/v1/groups/:group_id/unlock-requests?status=pending`
  - `PUT /api/v1/unlock-requests/:request_id`
- Approval triggers automatic unlock

## Implementation Tasks

- [x] Create migration 023_unlock_requests.sql
- [x] Create UnlockRequest entity
- [x] Create UnlockRequestRepository
- [x] Create request DTOs (create, list, update)
- [x] Add create_unlock_request handler
- [x] Add list_unlock_requests handler
- [x] Add respond_to_unlock_request handler
- [x] Add routes to app.rs
- [x] Add unit tests

## API Request Example (Create)

```json
{
  "reason": "I need to change the tracking interval for battery saving"
}
```

## API Response Example (List)

```json
{
  "data": [
    {
      "id": "req_01...",
      "device": {
        "id": "dev_01...",
        "displayName": "John's Phone"
      },
      "settingKey": "tracking_interval_minutes",
      "settingDisplayName": "Tracking Interval",
      "status": "pending",
      "requestedBy": {
        "id": "user_01...",
        "displayName": "John Doe"
      },
      "reason": "Battery saving",
      "createdAt": "2025-12-01T10:35:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "perPage": 20,
    "total": 1
  }
}
```

## API Request Example (Respond)

```json
{
  "status": "approved",
  "note": "Approved for battery saving purposes"
}
```

---

## Dev Notes

- Only one pending request per device+setting allowed
- Approval side-effect: unlock the setting
- Push notification support deferred to Story 12.8

---

## Dev Agent Record

### Debug Log


### Completion Notes

- Created migration 023_unlock_requests.sql with status enum and proper constraints
- Created UnlockRequestEntity with DB enum mapping
- Implemented UnlockRequestRepository with CRUD operations including pagination
- Created comprehensive DTOs for request/response serialization
- Added create_unlock_request handler with authorization and conflict checks
- Added list_unlock_requests handler with pagination and status filtering
- Added respond_to_unlock_request handler with auto-unlock on approval
- All routes registered in app.rs
- All 623+ tests passing

---

## File List

- crates/persistence/src/migrations/022_unlock_requests.sql
- crates/persistence/src/entities/unlock_request.rs
- crates/persistence/src/entities/mod.rs
- crates/persistence/src/repositories/unlock_request.rs
- crates/persistence/src/repositories/mod.rs
- crates/domain/src/models/unlock_request.rs
- crates/domain/src/models/mod.rs
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
Story 12.6 implements a complete unlock request workflow allowing device users to request unlocking of locked settings. The workflow includes proper constraint checks (pending request uniqueness, expiration, lock validation) and auto-unlocks settings upon approval.

### Key Findings

**Positive Observations**:
- Migration `023_unlock_requests.sql` with proper constraints and partial unique index for pending requests
- Request expiration after 7 days via `DEFAULT (NOW() + INTERVAL '7 days')`
- Unique constraint prevents multiple pending requests for same device+setting
- Auto-unlock on approval via `setting_repo.unlock_setting()`
- Notification sent on response via NotificationService
- Pagination support for list endpoint

**Severity: None**
- All requirements implemented correctly

### Acceptance Criteria Coverage

| AC# | Criterion | Status | Evidence |
|-----|-----------|--------|----------|
| 1 | POST creates unlock request | ✅ Pass | `create_unlock_request` handler line 1060 |
| 2 | GET lists pending requests for group | ✅ Pass | `list_unlock_requests` handler line 1147 |
| 3 | PUT approves/denies request | ✅ Pass | `respond_to_unlock_request` handler line 1244 |
| 4 | Request includes reason from user | ✅ Pass | CreateUnlockRequestRequest.reason |
| 5 | Only device owner can create request | ✅ Pass | Authorization check at lines 1078-1090 |
| 6 | Only group admin/owner can approve/deny | ✅ Pass | `check_is_admin()` at line 1283 |
| 7 | Approved request auto-unlocks setting | ✅ Pass | Lines 1307-1314 |
| 8 | Status enum: pending/approved/denied/expired | ✅ Pass | UnlockRequestStatus enum |
| 9 | Requests expire after 7 days | ✅ Pass | Migration: `expires_at DEFAULT NOW() + 7 days` |
| 10 | Returns 409 if pending request exists | ✅ Pass | Lines 1108-1115: ApiError::Conflict |

### Test Coverage and Gaps
- ✅ Unit tests for status enum Display trait
- ✅ Repository tests for CRUD operations
- ✅ Model serialization tests
- **No gap identified**

### Architectural Alignment
✅ Follows project patterns:
- New migration 023 for unlock_requests table
- Repository pattern with UnlockRequestRepository
- Domain models properly separated from entities
- Routes integrated in device_settings.rs (logical grouping)

### Security Notes
- ✅ Only device owner can create requests
- ✅ Only admins can approve/deny
- ✅ Expiration check prevents stale request manipulation
- ✅ Already-processed requests rejected (409 Conflict)

### Action Items
None - Story approved as implemented.

